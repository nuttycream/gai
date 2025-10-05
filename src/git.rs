use anyhow::{Result, bail};
use git2::{
    DiffHunk, DiffLine, DiffOptions, Repository, StatusOptions,
};
use std::{collections::HashMap, path::Path};

use crate::{config::Config, response::Commit};

pub struct GaiGit {
    pub file_diffs: HashMap<String, Vec<HunkDiff>>,
    pub truncated_files: Vec<String>,

    repo: Repository,
}

#[derive(Debug, Clone)]
pub struct HunkDiff {
    /// example key (header)
    /// @@ -12,8 +12,9 @@
    /// since raw line numbers
    /// may be inconsistent
    pub header: String,

    pub line_diffs: Vec<LineDiff>,
}

#[derive(Debug, Clone)]
pub struct LineDiff {
    pub diff_type: DiffType,
    pub content: String,
}

/// taken from diffline::origin
#[derive(Clone, Default, Debug, Eq, Hash, PartialEq)]
pub enum DiffType {
    #[default]
    Unchanged,
    Additions,
    Deletions,
}

impl GaiGit {
    /// todo: this could fail on an unitialized directory
    /// for now, im not gonna handle those and we
    /// just straight up panic if we failed to open
    pub fn new(repo_path: &str) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        let mut options = StatusOptions::new();

        options.include_untracked(true);

        if repo.statuses(Some(&mut options))?.is_empty() {
            bail!("no diffs");
        }

        Ok(GaiGit {
            repo,
            file_diffs: HashMap::new(),
            truncated_files: Vec::new(),
        })
    }

    pub fn create_diffs(
        &mut self,
        files_to_truncate: &[String],
    ) -> Result<(), git2::Error> {
        // start this puppy up
        let mut opts = DiffOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .enable_fast_untracked_dirs(true);

        let repo = &self.repo;

        let head = repo.head()?.peel_to_tree()?;
        let diff =
            repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;

        // after moving this from main, i still need to track
        // the hunks, so i can group them per file.
        // this may not be needed in the future
        // once we add some sort of way to stage
        // specific hunks similar to git add -p
        let mut unique_hunks = HashMap::new();

        diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            /* match delta.status() {
                Delta::Added => todo!(),
                Delta::Deleted => todo!(),
                Delta::Modified => todo!(),
                Delta::Renamed => todo!(),
                Delta::Copied => todo!(),
                Delta::Ignored => todo!(),
                Delta::Untracked => todo!(),
                Delta::Typechange => todo!(),
                Delta::Unreadable => todo!(),
                Delta::Conflicted => todo!(),
                ignore unmodified
                _ => {}
            } */

            let path = delta
                .new_file()
                .path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            if files_to_truncate.iter().any(|f| path.ends_with(f)) {
                self.truncated_files.push(path.clone());
            }

            let diff_hunks =
                unique_hunks.entry(path).or_insert_with(Vec::new);

            process_file_diff(diff_hunks, &hunk, &line);

            true
        })?;

        self.file_diffs = unique_hunks;

        // handle untracked files here
        // would create a sep func, but lesdodis for now
        // also i had this before, forgot to re-add after rewrite
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        let statuses = self.repo.statuses(Some(&mut status_opts))?;

        for entry in statuses.iter() {
            if entry.status().contains(git2::Status::WT_NEW) {
                let path = entry.path().unwrap();

                if files_to_truncate.iter().any(|f| path.ends_with(f))
                {
                    self.truncated_files.push(path.to_owned());
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(path) {
                    let lines: Vec<LineDiff> = content
                        .lines()
                        .map(|line| LineDiff {
                            diff_type: DiffType::Additions,
                            content: format!("{}\n", line),
                        })
                        .collect();

                    self.file_diffs.insert(
                        path.to_string(),
                        vec![HunkDiff {
                            // just set header as a new file,
                            // hopefully thats enough context
                            // for an llm
                            header: format!(
                                "new file {}",
                                lines.len()
                            ),
                            line_diffs: lines,
                        }],
                    );
                }
            }
        }

        Ok(())
    }

    pub fn get_file_diffs_as_str(&self) -> HashMap<String, String> {
        let mut file_diffs = HashMap::new();
        for (path, hunks) in &self.file_diffs {
            let mut diff_str = String::new();

            for hunk in hunks {
                diff_str.push_str(&hunk.header);
                diff_str.push('\n');

                for line in &hunk.line_diffs {
                    let prefix = match line.diff_type {
                        DiffType::Unchanged => ' ',
                        DiffType::Additions => '+',
                        DiffType::Deletions => '-',
                    };
                    diff_str.push(prefix);
                    diff_str.push_str(&line.content);
                }
                diff_str.push('\n');
            }
            if !diff_str.trim().is_empty() {
                file_diffs.insert(path.to_owned(), diff_str);
            }
        }

        file_diffs
    }

    pub fn apply_commits(&self, commits: &[Commit], cfg: &Config) {
        //println!("{:#?}", self.commits);
        for commit in commits {
            self.commit(commit, cfg);
        }
    }

    fn commit(&self, commit: &Commit, cfg: &Config) {
        let mut index = self.repo.index().unwrap();

        index.clear().unwrap();

        if let Ok(head) = self.repo.head()
            && let Ok(tree) = head.peel_to_tree()
        {
            index.read_tree(&tree).unwrap();
        }

        // staging
        for path in &commit.files {
            let path = Path::new(&path);
            let status = self.repo.status_file(path).unwrap();

            if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::WT_NEW)
            {
                index.add_path(path).unwrap();
            }
        }

        index.write().unwrap();

        let tree_oid = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_oid).unwrap();

        let parent_commit = match self.repo.revparse_single("HEAD") {
            Ok(obj) => Some(obj.into_commit().unwrap()),
            // ignore first commit
            Err(_) => None,
        };

        let mut parents = Vec::new();
        if let Some(parent) = parent_commit.as_ref() {
            parents.push(parent);
        }

        let sig = self.repo.signature().unwrap();
        let commit_msg = &commit.get_commit_message(cfg);

        self.repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                commit_msg,
                &tree,
                &parents[..],
            )
            .unwrap();
    }

    // won't bother with this for now
    //fn new_branch(&self, commit: &Commit) {}
}

fn process_file_diff(
    diff_hunks: &mut Vec<HunkDiff>,
    hunk: &Option<DiffHunk>,
    line: &DiffLine,
) {
    if let Some(h) = hunk {
        let header = str::from_utf8(h.header())
            .unwrap_or("not a valid utf8 header from hunk")
            .to_owned();
        let content = str::from_utf8(line.content())
            .unwrap_or("not a valid utf8 line from hunk")
            .to_owned();

        let diff_type = match line.origin() {
            '+' => DiffType::Additions,
            '-' => DiffType::Deletions,
            ' ' => DiffType::Unchanged,
            _ => return,
        };

        let diff_line = LineDiff { diff_type, content };

        // instead of storing the different types.
        // we can just push line diffs in a clear order
        // if i want to filter it out, i can do that
        // later, this should just care about the diff itself
        match diff_hunks.iter_mut().find(|h| h.header == header) {
            Some(existing) => existing.line_diffs.push(diff_line),
            None => {
                diff_hunks.push(HunkDiff {
                    header,
                    line_diffs: vec![diff_line],
                });
            }
        }
    }
}
