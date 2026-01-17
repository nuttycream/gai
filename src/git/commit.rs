use git2::{Diff, ErrorCode, Oid, Repository, Signature};

use super::{
    status::{FileStatus, StatusItemType},
    utils::get_head_repo,
};

#[derive(Debug)]
pub struct GitCommit {
    pub files: Vec<String>,
    pub hunk_ids: Vec<String>,
    pub message: String,
}

/// struct containing a new and an old version
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct OldNew<T> {
    /// The old version
    pub old: T,
    /// The new version
    pub new: T,
}

pub fn commit(
    repo: &Repository,
    commit: &GitCommit,
) -> anyhow::Result<Oid> {
    let mut index = repo.index()?;

    let signature = repo.signature()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let parents = if let Ok(id) = get_head_repo(repo) {
        vec![repo.find_commit(id)?]
    } else {
        Vec::new()
    };

    let parents = parents
        .iter()
        .collect::<Vec<_>>();

    let oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit.message,
        &tree,
        parents.as_slice(),
    )?;

    Ok(oid)
}

pub fn get_commit_files(
    repo: &Repository,
    id: Oid,
    other: Option<Oid>,
) -> anyhow::Result<Vec<FileStatus>> {
    let diff = if let Some(other) = other {
        get_compare_commits_diff(
            repo,
            sort_commits(repo, (id, other))?,
        )?
    } else {
        get_commit_diff(repo, id)?
    };

    let res = diff
        .deltas()
        .map(|delta| {
            let status = StatusItemType::from(delta.status());

            FileStatus {
                path: delta
                    .new_file()
                    .path()
                    .map(|p| {
                        p.to_str()
                            .unwrap_or("")
                            .to_string()
                    })
                    .unwrap_or_default(),
                status,
            }
        })
        .collect::<Vec<_>>();

    Ok(res)
}

/// get diff of a commit to its first parent
pub fn get_commit_diff<'a>(
    repo: &'a Repository,
    id: Oid,
) -> anyhow::Result<Diff<'a>> {
    let commit = repo.find_commit(id)?;
    let commit_tree = commit.tree()?;

    let parent = if commit.parent_count() > 0 {
        repo.find_commit(commit.parent_id(0)?)
            .ok()
            .and_then(|c| c.tree().ok())
    } else {
        None
    };

    let mut opts = git2::DiffOptions::new();

    opts.show_binary(true);

    let diff = repo.diff_tree_to_tree(
        parent.as_ref(),
        Some(&commit_tree),
        Some(&mut opts),
    )?;

    Ok(diff)
}

/// get diff of two arbitrary commits
fn get_compare_commits_diff(
    repo: &Repository,
    ids: OldNew<Oid>,
) -> anyhow::Result<Diff<'_>> {
    let commits = OldNew {
        old: repo.find_commit(ids.old)?,
        new: repo.find_commit(ids.new)?,
    };

    let trees = OldNew {
        old: commits.old.tree()?,
        new: commits.new.tree()?,
    };

    let mut opts = git2::DiffOptions::new();

    let diff: Diff<'_> = repo.diff_tree_to_tree(
        Some(&trees.old),
        Some(&trees.new),
        Some(&mut opts),
    )?;

    Ok(diff)
}

fn sort_commits(
    repo: &Repository,
    commits: (Oid, Oid),
) -> anyhow::Result<OldNew<Oid>> {
    if repo.graph_descendant_of(commits.0, commits.1)? {
        Ok(OldNew {
            old: commits.1,
            new: commits.0,
        })
    } else {
        Ok(OldNew {
            old: commits.0,
            new: commits.1,
        })
    }
}
