use std::{
    fmt::{self, format},
    path::Path,
};

use git2::{
    Delta, Oid, Repository, Status, StatusOptions, StatusShow,
};

/// status strategy when running
/// get_status
#[derive(
    Debug, Clone, Default, serde::Serialize, serde::Deserialize,
)]
pub enum StatusStrategy {
    /// only get status
    /// of working dir
    WorkingDir,
    /// only get status
    /// of what's currently staged
    Stage,
    /// both, this does not differentiate between
    /// the two, meaning wt and index are shown
    /// as one status
    #[default]
    Both,
}

#[derive(Debug, Default)]
pub struct GitStatus {
    pub branch_name: String,
    pub statuses: Vec<FileStatus>,
}

#[derive(Debug)]
pub struct FileStatus {
    pub path: String,
    pub status: StatusItemType,
}

#[derive(strum::Display, Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum StatusItemType {
    New,
    Modified,
    Deleted,
    Renamed,
    Typechange,
    Conflicted,
}

// opts.show
impl From<StatusStrategy> for StatusShow {
    fn from(s: StatusStrategy) -> Self {
        match s {
            StatusStrategy::WorkingDir => Self::Workdir,
            StatusStrategy::Stage => Self::Index,
            StatusStrategy::Both => Self::IndexAndWorkdir,
        }
    }
}

impl From<Status> for StatusItemType {
    fn from(s: Status) -> Self {
        if s.is_index_new() || s.is_wt_new() {
            Self::New
        } else if s.is_index_deleted() || s.is_wt_deleted() {
            Self::Deleted
        } else if s.is_index_renamed() || s.is_wt_renamed() {
            Self::Renamed
        } else if s.is_index_typechange() || s.is_wt_typechange() {
            Self::Typechange
        } else if s.is_conflicted() {
            Self::Conflicted
        } else {
            Self::Modified
        }
    }
}

impl From<Delta> for StatusItemType {
    fn from(d: Delta) -> Self {
        match d {
            Delta::Added => Self::New,
            Delta::Deleted => Self::Deleted,
            Delta::Renamed => Self::Renamed,
            Delta::Typechange => Self::Typechange,
            _ => Self::Modified,
        }
    }
}

// helper ONLY FOR LLM REQUESTS
// not for pretty print status
impl fmt::Display for GitStatus {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut s = String::new();

        let title = format!("Branch:{}\n", self.branch_name);
        s.push_str(&title);

        for git_status in &self.statuses {
            s.push_str(&format!(
                "{}:{}",
                git_status.status, git_status.path
            ));
            s.push('\n');
        }

        write!(f, "{}", s)
    }
}

pub fn is_workdir_clean(repo: &Repository) -> anyhow::Result<bool> {
    if repo.is_bare() && !repo.is_worktree() {
        return Ok(true);
    }

    let mut options = StatusOptions::default();
    options
        .show(StatusShow::Workdir)
        .update_index(true)
        .include_untracked(true)
        .renames_head_to_index(true)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut options))?;

    Ok(statuses.is_empty())
}

/// func to get stats of a specific commit
/// specifically, files changed and the inserts
/// deletions within em. meant to mimic commiting
/// completion output
/// FIXME: this breaks on a root commit
pub(crate) fn get_commit_stats(
    repo: &Repository,
    hash: &str,
) -> anyhow::Result<(String, usize, usize, usize)> {
    let oid = Oid::from_str(hash)?;

    let commit = repo.find_commit(oid)?;

    let tree = commit.tree()?;

    // not using find_parent_commit.
    // FIXME: this is assuming that this func
    // runs after committing, this will break
    // if this is the root commit
    let parent = commit
        .parent(0)?
        .tree()?;

    let diff =
        repo.diff_tree_to_tree(Some(&parent), Some(&tree), None)?;

    let stats = diff.stats()?;
    let branch_name = get_branch_name(repo)?;

    Ok((
        branch_name,
        stats.files_changed(),
        stats.insertions(),
        stats.deletions(),
    ))
}

pub fn get_status(
    repo: &Repository,
    strategy: &StatusStrategy,
) -> anyhow::Result<GitStatus> {
    let mut opts = StatusOptions::default();

    // filter
    opts.show(
        strategy
            .to_owned()
            .into(),
    );

    opts.update_index(true);
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);
    /* opts.renames_head_to_index(true);
    opts.renames_index_to_workdir(true); */

    let statuses = repo.statuses(Some(&mut opts))?;
    let branch_name = get_branch_name(repo)?;

    let mut statuses: Vec<FileStatus> = statuses
        .iter()
        .filter_map(|entry| {
            let status: StatusItemType = entry
                .status()
                .into();
            let path = entry
                .path()?
                .to_string();

            // for workdir renames entry.path returns the older path
            // this is a temp fix so we can get the
            // NEW path where the file actually exists
            /* let path = if status == StatusItemType::Renamed {
                entry
                    .index_to_workdir()?
                    .new_file()
                    .path()?
                    .to_string_lossy()
                    .to_string()
            } else {
                entry.path()?.to_string()
            }; */

            Some(FileStatus { path, status })
        })
        .collect();

    statuses
        .sort_by(|a, b| Path::new(&a.path).cmp(Path::new(&b.path)));

    Ok(GitStatus {
        branch_name,
        statuses,
    })
}

fn get_branch_name(repo: &Repository) -> anyhow::Result<String> {
    let binding = repo.head()?;
    let head = binding.shorthand();

    Ok(head
        .unwrap_or("HEAD")
        .to_string())
}
