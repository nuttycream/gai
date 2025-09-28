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
        //println!("{:#?}", self.commits);
        for commit in &self.commits {
            self.commit(commit);
        }
    }

    fn commit(&self, commit: &Commit) {
        let mut index = self.repo.index().unwrap();

        index.clear().unwrap();

        if let Ok(head) = self.repo.head() {
            if let Ok(tree) = head.peel_to_tree() {
                index.read_tree(&tree).unwrap();
            }
        }

        // staging
        for path in &commit.files {
            let path = Path::new(&path);
            let status = self.repo.status_file(path).unwrap();

            if status.contains(Status::WT_MODIFIED)
                || status.contains(Status::WT_NEW)
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

        self.repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!(
                    "{}: {}",
                    match commit.message.prefix {
                        _ => format!("{:?}", commit.message.prefix),
                    },
                    commit.message.message
                ),
                &tree,
                &parents[..],
            )
            .unwrap();
    }

    // won't bother with this for now
    //fn new_branch(&self, commit: &Commit) {}
}
