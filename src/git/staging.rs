use std::path::Path;

use crate::{
    ai::response::GaiCommit, config::Config, git::repo::GaiGit,
};

impl GaiGit {
    pub fn apply_commits(&self, commits: &[GaiCommit], cfg: &Config) {
        //println!("{:#?}", self.commits);
        for commit in commits {
            self.commit(commit, cfg);
        }
    }

    fn commit(&self, commit: &GaiCommit, cfg: &Config) {
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
}
