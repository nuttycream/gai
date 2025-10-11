use std::{path::Path, str::from_utf8};

use git2::{ApplyOptions, DiffOptions};

use crate::git::{commit::GaiCommit, repo::GaiGit};

impl GaiGit {
    pub fn apply_commits(&self, commits: &[GaiCommit]) {
        //println!("{:#?}", self.commits);
        for commit in commits {
            self.commit(commit);
        }
    }

    fn commit(&self, commit: &GaiCommit) {
        let mut index = self.repo.index().unwrap();

        index.clear().unwrap();

        if let Ok(head) = self.repo.head()
            && let Ok(tree) = head.peel_to_tree()
        {
            index.read_tree(&tree).unwrap();
        }

        if self.stage_hunks {
            for path in &commit.files {
                let path = Path::new(&path);
                let mut diff_opts = DiffOptions::new();
                diff_opts.pathspec(path);

                let diff = self
                    .repo
                    .diff_index_to_workdir(
                        Some(&index),
                        Some(&mut diff_opts),
                    )
                    .unwrap();

                let selected_headers = &commit.hunk_headers;
                let mut apply_opts = ApplyOptions::new();

                apply_opts.hunk_callback(|h| {
                    if let Some(hunk) = h {
                        let header = from_utf8(hunk.header())
                            .unwrap()
                            .trim()
                            .to_string();

                        let hunk_exists = selected_headers
                            .iter()
                            .any(|h| h.trim() == header);

                        return hunk_exists;
                    }
                    true
                });

                match self.repo.apply(
                    &diff,
                    git2::ApplyLocation::Index,
                    Some(&mut apply_opts),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("failed to apply commits:{}", e);
                    }
                }
            }
        } else {
            // staging per file
            for path in &commit.files {
                let path = Path::new(&path);
                let status = self.repo.status_file(path).unwrap();

                // todo: some changes will implement a combo
                // ex: modified + renamed
                // i think we need to explicitly handle those
                // maybe by storing it in a buffer of some sort
                if status.contains(git2::Status::WT_MODIFIED)
                    || status.contains(git2::Status::WT_NEW)
                {
                    index.add_path(path).unwrap();
                }
                if status.contains(git2::Status::WT_DELETED) {
                    index.remove_path(path).unwrap();
                }
                if status.contains(git2::Status::WT_TYPECHANGE) {
                    index.remove_path(path).unwrap();
                    index.add_path(path).unwrap();
                }
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

        let commit_msg = &commit.message;

        self.repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                &commit_msg,
                &tree,
                &parents[..],
            )
            .unwrap();
    }
}
