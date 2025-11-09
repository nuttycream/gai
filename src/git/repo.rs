use anyhow::{Result, bail};
use git2::{Repository, Status, StatusOptions};
use std::collections::HashMap;
use walkdir::WalkDir;

pub struct GaiGit {
    /// Diffs
    pub files: Vec<GaiFile>,

    /// git2 based Repo
    pub repo: Repository,

    pub status: GaiStatus,

    pub stage_hunks: bool,
    pub capitalize_prefix: bool,
    pub include_scope: bool,
}

/// helper to store paths for the files
/// marked along status
pub struct GaiStatus {
    pub s_new: Vec<String>,
    pub s_modified: Vec<String>,
    pub s_deleted: Vec<String>,
    // old -> new
    pub s_renamed: Vec<(String, String)>,

    // unstaged
    pub u_new: Vec<String>,
    pub u_modified: Vec<String>,
    pub u_deleted: Vec<String>,
    pub u_renamed: Vec<(String, String)>,
}

/// a sort of DiffDelta struct
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

        let status = Self::build_status(&repo)?;

        Ok(GaiGit {
            repo,
            files: Vec::new(),
            status,
            stage_hunks,
            capitalize_prefix,
            include_scope,
        })
    }

    fn build_status(repo: &Repository) -> Result<GaiStatus> {
        let mut status_opts = StatusOptions::new();

        status_opts.include_untracked(true);

        let statuses = repo.statuses(Some(&mut status_opts))?;

        let mut status = GaiStatus {
            s_new: Vec::new(),
            s_modified: Vec::new(),
            s_deleted: Vec::new(),
            s_renamed: Vec::new(),
            u_new: Vec::new(),
            u_modified: Vec::new(),
            u_deleted: Vec::new(),
            u_renamed: Vec::new(),
        };

        for e in statuses.iter() {
            if e.status().contains(Status::IGNORED) {
                continue;
            }

            let path = e.path().unwrap_or("").to_string();

            if e.status().contains(Status::INDEX_NEW) {
                status.s_new.push(path.to_owned());
            }

            if e.status().contains(Status::INDEX_MODIFIED) {
                status.s_modified.push(path.to_owned());
            }

            if e.status().contains(Status::INDEX_DELETED) {
                status.s_deleted.push(path.to_owned());
            }

            if e.status().contains(Status::INDEX_RENAMED)
                && let Some(diff) = e.head_to_index()
            {
                let old_path = diff
                    .old_file()
                    .path()
                    .and_then(|p| p.to_str())
                    .unwrap_or("");
                let new_path = diff
                    .new_file()
                    .path()
                    .and_then(|p| p.to_str())
                    .unwrap_or("");
                status.s_renamed.push((
                    old_path.to_string(),
                    new_path.to_string(),
                ));
            }

            if e.status().contains(Status::WT_NEW) {
                status.u_new.push(path.to_owned());
            }

            if e.status().contains(Status::WT_MODIFIED) {
                status.u_modified.push(path.to_owned());
            }

            if e.status().contains(Status::WT_DELETED) {
                status.u_deleted.push(path.to_owned());
            }

            if e.status().contains(Status::WT_RENAMED)
                && let Some(diff) = e.index_to_workdir()
            {
                let old = diff
                    .old_file()
                    .path()
                    .and_then(|p| p.to_str())
                    .unwrap_or("");

                let new = diff
                    .new_file()
                    .path()
                    .and_then(|p| p.to_str())
                    .unwrap_or("");

                status
                    .u_renamed
                    .push((old.to_owned(), new.to_owned()));
            }
        }

        Ok(status)
    }

    pub fn get_repo_tree(&self) -> String {
        let repo_root =
            self.repo.workdir().ok_or("not a workdir").unwrap();

        let mut repo_tree = String::new();

        for e in WalkDir::new(repo_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(rel_path) = e.path().strip_prefix(repo_root)
                && !self.repo.status_should_ignore(rel_path).unwrap()
            {
                repo_tree
                    .push_str(&format!("{}\n", rel_path.display()));
            }
        }

        repo_tree
    }

    pub fn get_repo_status_as_str(&self) -> String {
        let mut staged = String::new();
        let mut unstaged = String::new();

        for path in &self.status.s_new {
            staged.push_str(&format!("A  {}\n", path));
        }

        for path in &self.status.s_modified {
            staged.push_str(&format!("M  {}\n", path));
        }

        for path in &self.status.s_deleted {
            staged.push_str(&format!("D  {}\n", path));
        }

        for (old, new) in &self.status.s_renamed {
            staged.push_str(&format!("R  {} -> {}\n", old, new));
        }

        for path in &self.status.u_new {
            unstaged.push_str(&format!("? {}\n", path));
        }

        for path in &self.status.u_modified {
            unstaged.push_str(&format!("M {}\n", path));
        }

        for path in &self.status.u_deleted {
            unstaged.push_str(&format!("D {}\n", path));
        }

        for (old, new) in &self.status.u_renamed {
            unstaged.push_str(&format!("R {} -> {}\n", old, new));
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
