use std::collections::HashMap;

use git2::{Diff, Oid, Repository};

use super::{
    diffs::{FileDiff, HunkId},
    staging::{StagingStrategy, stage_all, stage_file, stage_hunks},
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

pub fn apply_commits(
    repo: &Repository,
    git_commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_stragey: &StagingStrategy,
) -> anyhow::Result<()> {
    //todo when we implement verbose logging
    // make sure we log the files, hunks etc
    // before we apply commits

    for git_commit in git_commits {
        match staging_stragey {
            StagingStrategy::AllFilesOneCommit => {
                stage_all(repo, ".")?;
                og_file_diffs.clear();
                commit(repo, git_commit)?;

                // return early
                return Ok(());
            }
            StagingStrategy::AtomicCommits
            | StagingStrategy::OneFilePerCommit => {
                for file in &git_commit.files {
                    stage_file(repo, file)?;
                    // remove if status matches
                    //remove_file(&git.repo, file)?;
                    og_file_diffs.retain(|f| f.path != file.as_str());
                }
            }
            StagingStrategy::Hunks => {
                // this commit should define its hunkids
                // to stage like:
                // commit 1: src/main.rs:0, src/main.rs:1 etc
                // group hunks based on the file paths
                // iterate over each file
                // find what hunks to stage
                // pass it into stage_hunks
                // stage_hunks should be able to apply
                // only the hunks it gets from here

                // file_path and a list of hunk indecises
                let mut files: HashMap<String, Vec<usize>> =
                    HashMap::new();

                // group hunks to their file_paths
                for hunk in &git_commit.hunk_ids {
                    let hunk_id = HunkId::try_from(hunk.as_str())?;
                    files
                        .entry(hunk_id.path.clone())
                        .or_default()
                        .push(hunk_id.index);
                }

                // now process each file
                for (file_path, hunk_ids) in files {
                    // find the original file associated
                    // with this from the og database
                    let og_file_diff = og_file_diffs
                        .iter()
                        .find(|f| f.path == file_path)
                        .ok_or({
                            anyhow::anyhow!(
                                "{} is not in the og_file_diffs",
                                file_path
                            )
                        })?;

                    if og_file_diff.untracked {
                        stage_file(repo, &file_path)?;
                        og_file_diffs.retain(|f| f.path != file_path);
                        continue;
                    }

                    // get relevant hunk ids
                    let hunks = super::diffs::find_file_hunks(
                        og_file_diff,
                        hunk_ids,
                    )?;

                    // stage hunks relevant to this file ONLY
                    let used = stage_hunks(repo, &file_path, &hunks)?;

                    super::diffs::remove_hunks(
                        og_file_diffs,
                        &file_path,
                        &used,
                    );
                }
            }
        }

        commit(repo, git_commit)?;
    }

    for file in og_file_diffs {
        for hunk in &file.hunks {
            println!("hunk [{}:{}] not applied", file.path, hunk.id);
        }
    }

    Ok(())
}

fn commit(
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
