use anyhow::{Result, bail};
use git2::{Repository, StatusOptions};
use std::collections::HashMap;
use walkdir::WalkDir;

// todo
// here's the plan
// gonna need to rewrite some of this
// after testing i found out that hunk headers
// may change, after committing
// but its dependent on whether or not
// the LLM organizses it in a way that avoids changing the
// lines in a hunk. from testing, apply_opts hunk callback
// will fail on certain hunks and skip it, because
// it may not exist because it was changed
//
// so my plan is to create a a different way to track
// hunks, storing a hash of that hunks content
// then using that to compare with content from the hunks as we
// go through them
// if that hunk matches the hash that belongs to this commit
// then apply it.
// hopefully this method works
pub struct GaiGit {
    pub files: Vec<GaiFile>,
    pub repo: Repository,

    pub stage_hunks: bool,

    pub capitalize_prefix: bool,
    pub include_scope: bool,
}

#[derive(Debug)]
pub struct GaiFile {
    pub path: String,
    pub should_truncate: bool,
    pub hunks: Vec<HunkDiff>,
}

#[derive(Debug, Clone)]
pub struct HunkDiff {
    /// example key (header)
    /// @@ -12,8 +12,9 @@
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
    pub fn new(
        stage_hunks: bool,
        capitalize_prefix: bool,
        include_scope: bool,
    ) -> Result<Self> {
        let repo = Repository::open_from_env()?;
        let mut options = StatusOptions::new();

        options.include_untracked(true);

        if repo.statuses(Some(&mut options))?.is_empty() {
            bail!("no diffs");
        }

        Ok(GaiGit {
            repo,
            files: Vec::new(),
            stage_hunks,
            capitalize_prefix,
            include_scope,
        })
    }

    pub fn get_repo_tree(&self) -> String {
        let repo_root =
            self.repo.workdir().ok_or("not a workdir").unwrap();

        let mut repo_tree = String::new();

        for entry in WalkDir::new(repo_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(rel_path) = entry.path().strip_prefix(repo_root)
                && !self.repo.status_should_ignore(rel_path).unwrap()
            {
                repo_tree
                    .push_str(&format!("{}\n", rel_path.display()));
            }
        }

        repo_tree
    }

    pub fn get_repo_status(&self) -> String {
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        let statuses = self.repo
            .statuses(Some(&mut status_opts))
            .expect("somehow failed to get statuses the second time around");

        let mut staged = String::new();
        let mut unstaged = String::new();

        for entry in statuses
            .iter()
            .filter(|e| e.status() != git2::Status::CURRENT)
        {
            let istatus = match entry.status() {
                s if s.contains(git2::Status::INDEX_NEW) => 'A',
                s if s.contains(git2::Status::INDEX_MODIFIED) => 'M',
                s if s.contains(git2::Status::INDEX_DELETED) => 'D',
                s if s.contains(git2::Status::INDEX_RENAMED) => 'R',
                s if s.contains(git2::Status::INDEX_TYPECHANGE) => {
                    'T'
                }
                _ => ' ',
            };

            let wstatus = match entry.status() {
                s if s.contains(git2::Status::WT_NEW) => '?',
                s if s.contains(git2::Status::WT_MODIFIED) => 'M',
                s if s.contains(git2::Status::WT_DELETED) => 'D',
                s if s.contains(git2::Status::WT_RENAMED) => 'R',
                s if s.contains(git2::Status::WT_TYPECHANGE) => 'T',
                _ => ' ',
            };

            if entry.status().contains(git2::Status::IGNORED) {
                continue;
            }

            let path = if let Some(diff) = entry.head_to_index() {
                diff.new_file().path()
            } else if let Some(diff) = entry.index_to_workdir() {
                diff.old_file().path()
            } else {
                None
            };

            if let Some(path) = path {
                let path_str = path.display().to_string();

                if istatus != ' ' {
                    staged.push_str(&format!(
                        "{}  {}\n",
                        istatus, path_str
                    ));
                }

                if wstatus != ' ' {
                    unstaged.push_str(&format!(
                        " {} {}\n",
                        wstatus, path_str
                    ));
                }
            }
        }

        let mut status_str = String::new();

        if !staged.is_empty() {
            status_str.push_str("Staged:\n");
            status_str.push_str(&staged);
            status_str.push('\n');
        }

        if !unstaged.is_empty() {
            status_str.push_str("Unstaged:\n");
            status_str.push_str(&unstaged);
        }

        status_str
    }

    pub fn get_file_diffs_as_str(&self) -> HashMap<String, String> {
        let mut file_diffs = HashMap::new();
        for gai_file in &self.files {
            let mut diff_str = String::new();
            if gai_file.should_truncate {
                diff_str.push_str("Truncated File");
                file_diffs.insert(gai_file.path.to_owned(), diff_str);
                continue;
            }

            for (i, hunk) in gai_file.hunks.iter().enumerate() {
                diff_str.push_str(&format!(
                    "Hunk_id[{}:{}]\n",
                    gai_file.path, i
                ));
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
                file_diffs.insert(gai_file.path.to_owned(), diff_str);
            }
        }

        file_diffs
    }
}
