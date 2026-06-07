use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
    path::Path,
};

use git2::{BlameOptions, Commit, Error, Oid, Repository};

use crate::git::{errors::GitError, utils::get_head_repo};

/// A `BlameHunk` contains all the information that will be shown to the user.
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct BlameHunk {
    pub commit_id: Oid,
    pub author: String,
    pub time: i64,
    /// `git2::BlameHunk::final_start_line` returns 1-based indices, but
    /// `start_line` is 0-based because the `Vec` storing the lines starts at
    /// index 0.
    pub start_line: usize,
    pub end_line: usize,
}

/// A `BlameFile` represents a collection of lines. This is targeted at how the
/// data will be used by the UI.
#[derive(Clone, Debug)]
pub struct FileBlame {
    pub commit_id: Oid,
    pub path: String,
    pub lines: Vec<(Option<BlameHunk>, String)>,
}

/// TODO: must fix for directories including .
pub fn blame_file(
    repo: &Repository,
    file_path: &str,
    commit_id: Option<String>,
) -> anyhow::Result<FileBlame> {
    let commit_id = if let Some(commit_id) = commit_id {
        commit_id.to_string()
    } else {
        get_head_repo(repo)?.to_string()
    };

    let spec = format!("{}:{}", commit_id, file_path);

    let object = repo.revparse_single(&spec)?;
    let blob = repo.find_blob(object.id())?;

    if blob.is_binary() {
        return Err(GitError::NoBlameOnBinary.into());
    }

    let commit_id = Oid::from_str(&commit_id)?;
    let mut opts = BlameOptions::new();
    opts.newest_commit(commit_id);

    let blame =
        repo.blame_file(Path::new(file_path), Some(&mut opts))?;

    let reader = BufReader::new(blob.content());

    let unique_commit_ids: HashSet<_> = blame
        .iter()
        .map(|hunk| hunk.final_commit_id())
        .collect();
    let mut commit_ids = Vec::with_capacity(unique_commit_ids.len());
    commit_ids.extend(unique_commit_ids);

    let commit_infos = get_commits_info(repo, &commit_ids)?;
    let unique_commit_infos: HashMap<_, _> = commit_infos
        .iter()
        .map(|commit_info| (commit_info.id, commit_info))
        .collect();

    let lines: Vec<(Option<BlameHunk>, String)> = reader
        .lines()
        .enumerate()
        .map(|(i, line)| {
            // Line indices in a `FileBlame` are 1-based.
            let corresponding_hunk = blame.get_line(i + 1);

            if let Some(hunk) = corresponding_hunk {
                let commit_id = hunk.final_commit_id();
                // Line indices in a `BlameHunk` are 1-based.
                let start_line = hunk
                    .final_start_line()
                    .saturating_sub(1);
                let end_line =
                    start_line.saturating_add(hunk.lines_in_hunk());

                if let Some(commit_info) =
                    unique_commit_infos.get(&commit_id)
                {
                    let hunk = BlameHunk {
                        commit_id,
                        author: commit_info
                            .author
                            .clone(),
                        time: commit_info.time,
                        start_line,
                        end_line,
                    };

                    return (
                        Some(hunk),
                        line.unwrap_or_else(|_| String::new()),
                    );
                }
            }

            (None, line.unwrap_or_else(|_| String::new()))
        })
        .collect();

    let file_blame = FileBlame {
        commit_id,
        path: file_path.into(),
        lines,
    };

    Ok(file_blame)
}

#[derive(Debug, Clone)]
struct CommitInfo {
    /// message
    pub message: String,
    /// time
    pub time: i64,
    /// author
    pub author: String,
    /// oid
    pub id: Oid,
}

fn get_commits_info(
    repo: &Repository,
    ids: &[Oid],
) -> anyhow::Result<Vec<CommitInfo>> {
    let commits = ids
        .iter()
        .map(|id| repo.find_commit(*id))
        .collect::<std::result::Result<Vec<Commit>, Error>>()?
        .into_iter();

    let res = commits
        .map(|c: Commit| {
            let message = c
                .message()
                .unwrap_or_default()
                .to_string();

            let author = c
                .author()
                .name()
                .unwrap_or("unknown")
                .to_string();

            CommitInfo {
                message,
                author,
                time: c.time().seconds(),
                id: c.id(),
            }
        })
        .collect::<Vec<_>>();

    Ok(res)
}
