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
                OpType::CommitChanges => self.commit(),
                OpType::NewBranch => self.new_branch(),
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

    fn commit(&self) {}

    fn new_branch(&self) {}
}
