use std::path::Path;

use git2::{Repository, Status};

use crate::response::Commit;

pub struct GitOps<'repo> {
    commits: Vec<Commit>,
    repo: &'repo Repository,
}

impl<'repo> GitOps<'repo> {
    pub fn init(
        ops: Vec<Commit>,
        repo: &'repo Repository,
    ) -> GitOps<'repo> {
        GitOps { commits: ops, repo }
    }

    pub fn apply_ops(&self) {
        println!("{:#?}", self.commits);
        for commit in &self.commits {
            self.commit(commit);
        }
    }

    fn commit(&self, commit: &Commit) {
        let mut index = self.repo.index().unwrap();

        // staging
        for path in &commit.files {
            let path = Path::new(&path);
            let status = self.repo.status_file(path).unwrap();

            if status.contains(Status::WT_MODIFIED)
                || status.contains(Status::WT_NEW)
            {
                let _ = index.add_path(path);
            }
        }

        let tree_oid = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_oid).unwrap();

        let parent_commit = match self.repo.revparse_single("HEAD") {
            Ok(obj) => Some(obj.into_commit().unwrap()),
            Err(e) => panic!("parent commit err: {e}"),
        };

        let mut parents = Vec::new();
        if parent_commit.is_some() {
            parents.push(parent_commit.as_ref().unwrap());
        }

        let sig = self.repo.signature().unwrap();
        self.repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                &commit.message.message,
                &tree,
                &parents[..],
            )
            .unwrap();
    }

    // won't bother with this for now
    //fn new_branch(&self, commit: &Commit) {}
}
