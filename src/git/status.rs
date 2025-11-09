use anyhow::Result;
use git2::{Repository, Status, StatusOptions};

use crate::git::repo::{GaiGit, GaiStatus};

impl GaiGit {
    pub fn build_status(
        repo: &Repository,
    ) -> Result<super::repo::GaiStatus> {
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

    pub fn staged_len(&self) -> usize {
        let s = &self.status;

        s.s_new.len()
            + s.s_modified.len()
            + s.s_deleted.len()
            + s.s_renamed.len()
    }

    pub fn unstaged_len(&self) -> usize {
        let s = &self.status;

        s.u_new.len()
            + s.u_modified.len()
            + s.u_deleted.len()
            + s.u_renamed.len()
    }
}
