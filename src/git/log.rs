use chrono::{DateTime, Utc};
use git2::Oid;
use std::fmt;

use super::{
    GitRepo,
    commit::{get_commit_diff, get_commit_files},
    diffs::{Diffs, raw_diff_to_file_diff},
};

#[derive(Debug, Default)]
pub struct Logs {
    pub git_logs: Vec<GitLog>,
}

/// represents a git commit in git logs
/// technically redudant, might have to
/// remove later
#[derive(Clone, Debug, Default)]
pub struct GitLog {
    pub prefix: Option<String>,
    pub breaking: bool,
    pub scope: Option<String>,
    pub header: Option<String>,
    pub body: Option<String>,

    // raw git commit message
    // used when we could not parse
    // prefix, scope, or header
    pub raw: String,

    /// might deprecate in favor of
    /// raw timestamp
    pub date: String,

    pub author: String,
    pub commit_hash: String,

    /// filled with get_commit_files
    pub files: Vec<String>,

    pub diffs: Diffs,
}

// parse a possible conventional commit
// with the format: prefix(scope)!: header
impl From<&[u8]> for GitLog {
    fn from(value: &[u8]) -> Self {
        let raw = String::from_utf8(value.to_owned()).unwrap_or_else(
            |_| "Failed to convert msg from utf8".to_owned(),
        );

        let first_line = raw
            .lines()
            .next()
            .unwrap_or("");
        let body = raw
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");

        let body = if body
            .trim()
            .is_empty()
        {
            None
        } else {
            Some(
                body.trim()
                    .to_string(),
            )
        };

        if let Some(colon_pos) = first_line.find(':') {
            let prefix_part = &first_line[..colon_pos];

            let header = first_line[colon_pos + 1..]
                .trim()
                .to_string();

            let header = if header.is_empty() {
                None
            } else {
                Some(header)
            };

            let breaking = prefix_part.contains('!');
            let prefix_part = prefix_part.replace('!', "");

            if let (Some(paren_start), Some(paren_end)) =
                (prefix_part.find('('), prefix_part.find(')'))
            {
                let prefix = prefix_part[..paren_start]
                    .trim()
                    .to_string();
                let scope = prefix_part[paren_start + 1..paren_end]
                    .trim()
                    .to_string();

                GitLog {
                    prefix: if prefix.is_empty() {
                        None
                    } else {
                        Some(prefix)
                    },
                    scope: if scope.is_empty() {
                        None
                    } else {
                        Some(scope)
                    },
                    breaking,
                    header,
                    body,
                    raw,
                    ..Default::default()
                }
            } else {
                let prefix = prefix_part
                    .trim()
                    .to_string();

                GitLog {
                    prefix: if prefix.is_empty() {
                        None
                    } else {
                        Some(prefix)
                    },
                    scope: None,
                    breaking,
                    header,
                    body,
                    raw,
                    ..Default::default()
                }
            }
        } else {
            // return raw if not a conventional commit standard
            // though, raw should always be filled
            GitLog {
                raw,
                ..Default::default()
            }
        }
    }
}

impl fmt::Display for Logs {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut s = String::new();

        for log in &self.git_logs {
            s.push_str(&format!("Author: {}", &log.author));
            s.push('\n');
            s.push_str(&format!("Message: {}", &log.raw));
            s.push('\n');
        }

        write!(f, "{}", s)
    }
}

// for displaying in print::log
// ideally with conventional
// commit components otherwise... raw
impl From<GitLog> for String {
    fn from(v: GitLog) -> Self {
        match (&v.prefix, &v.scope, &v.header) {
            (Some(prefix), Some(scope), Some(header)) => {
                let breaking = if v.breaking { "!" } else { "" };
                format!(
                    "{}({}){}: {}",
                    prefix, scope, breaking, header
                )
            }
            (Some(prefix), None, Some(header)) => {
                let breaking = if v.breaking { "!" } else { "" };
                format!("{}{}: {}", prefix, breaking, header)
            }
            (Some(prefix), Some(scope), None) => {
                let breaking = if v.breaking { "!" } else { "" };
                format!("{}({}){}", prefix, scope, breaking)
            }
            (Some(prefix), None, None) => {
                let breaking = if v.breaking { "!" } else { "" };
                format!("{}{}", prefix, breaking)
            }
            // only return the first line of the raw message
            // otherwise we'll get newlines and break
            // the display for logs
            _ => v
                .raw
                .lines()
                .next()
                .unwrap_or(&v.raw)
                .to_string(),
        }
    }
}

pub fn get_short_hash(git_log: &GitLog) -> String {
    git_log.commit_hash[..7.min(
        git_log
            .commit_hash
            .len(),
    )]
        .to_string()
}

// gets a single commit log from
// a specified commit hash string
// will always populate files, and
// file diff
pub fn get_log(
    git_repo: &GitRepo,
    commit: &str,
) -> anyhow::Result<GitLog> {
    let oid = Oid::from_str(commit)?;

    let repo = &git_repo.repo;

    let commit = repo.find_commit(oid)?;

    let mut log: GitLog = commit
        .message_bytes()
        .into();

    let author = commit.author();

    log.author = author
        .name()
        .unwrap_or("unknown author")
        .to_string();

    log.commit_hash = oid.to_string();
    log.date = DateTime::from_timestamp(
        author
            .when()
            .seconds(),
        0,
    )
    .map(|dt| {
        dt.format("%m/%d/%Y %H:%M:%S")
            .to_string()
    })
    .unwrap_or_default();

    log.files = get_commit_files(repo, oid, None)?
        .iter()
        .map(|f| f.path.to_string())
        .collect();

    for file in &log.files {
        let raw = get_commit_diff(repo, oid)?;
        let file_diff =
            raw_diff_to_file_diff(&raw, file, &git_repo.workdir)?;

        log.diffs
            .files
            .push(file_diff);
    }

    Ok(log)
}

#[allow(clippy::too_many_arguments)]
pub fn get_logs(
    git_repo: &GitRepo,
    files: bool,
    diffs: bool,
    count: usize,
    reverse: bool,
    from_hash: Option<&str>,
    to_hash: Option<&str>,
    since: Option<std::time::Duration>,
) -> anyhow::Result<Logs> {
    let repo = &git_repo.repo;
    let mut revwalk = repo.revwalk()?;

    if reverse {
        revwalk.set_sorting(git2::Sort::REVERSE)?;
    }

    match (from_hash, to_hash) {
        // range exists
        (Some(from), Some(to)) => {
            revwalk.push_range(&format!("{}..{}", from, to))?;
        }

        // from: hide it, walk from HEAD
        (Some(from), None) => {
            let oid = Oid::from_str(from)?;
            revwalk.hide(oid)?;
            revwalk.push_head()?;
        }

        // to: walk from that commit
        (None, Some(to)) => {
            let oid = Oid::from_str(to)?;
            revwalk.push(oid)?;
        }

        // if none just walk from HEAD
        (None, None) => {
            revwalk.push_head()?;
        }
    }

    let cont = if count == 0 { !0 } else { count };
    let revwalk = revwalk.take(cont);

    let mut git_logs = Vec::new();

    let last_time = if let Some(since) = since {
        Utc::now().timestamp() - since.as_secs() as i64
    } else {
        0
    };

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let timestamp = commit
            .author()
            .when()
            .seconds();

        if timestamp < last_time {
            break;
        }

        let mut log: GitLog = commit
            .message_bytes()
            .into();

        let author = commit.author();

        log.author = author
            .name()
            .unwrap_or("unknown author")
            .to_string();

        log.commit_hash = oid.to_string();
        log.date = DateTime::from_timestamp(
            author
                .when()
                .seconds(),
            0,
        )
        .map(|dt| {
            dt.format("%m/%d/%Y %H:%M:%S")
                .to_string()
        })
        .unwrap_or_default();

        if files {
            log.files = get_commit_files(repo, oid, None)?
                .iter()
                .map(|f| f.path.to_string())
                .collect();

            if diffs {
                for file in &log.files {
                    let raw = get_commit_diff(repo, oid)?;
                    let file_diff = raw_diff_to_file_diff(
                        &raw,
                        file,
                        &git_repo.workdir,
                    )?;

                    log.diffs
                        .files
                        .push(file_diff);
                }
            }
        }

        git_logs.push(log);
    }

    Ok(Logs { git_logs })
}
