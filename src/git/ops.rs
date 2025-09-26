use std::path::Path;

use git2::{Repository, Status};

use crate::response::{OpType, Operation};

pub struct GitOps<'repo> {
    ops: Vec<Operation>,
    repo: &'repo Repository,
}

impl<'repo> GitOps<'repo> {
    pub fn init(
        ops: Vec<Operation>,
        repo: &'repo Repository,
    ) -> GitOps<'repo> {
        GitOps { ops, repo }
    }

    pub fn apply_ops(&self) {
        println!("{:#?}", self.ops);
        for op in &self.ops {
            match op.op_type {
                OpType::StageFile => self.stage(op),
                OpType::CommitChanges => self.commit(op),
                OpType::NewBranch => self.new_branch(op),
            }
        }
    }

    fn stage(&self, op: &Operation) {
        let mut index = self.repo.index().unwrap();
        for path in &op.files {
            let path = Path::new(&path);
            let status = self.repo.status_file(path).unwrap();

            if status.contains(Status::WT_MODIFIED)
                || status.contains(Status::WT_NEW)
            {
                let _ = index.add_path(path);
            }
        }

        index.write().unwrap();
    }

    fn commit(&self, op: &Operation) {
        let mut index = self.repo.index().unwrap();

        let tree_oid = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_oid).unwrap();

        let parent_commit = match self.repo.revparse_single("HEAD") {
            Ok(obj) => Some(obj.into_commit().unwrap()),
            Err(e) => panic!("parent commit err"),
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
                &op.message.message,
                &tree,
                &parents[..],
            )
            .unwrap();
    }

    fn new_branch(&self, op: &Operation) {}
}
