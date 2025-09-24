use git2::Repository;

use crate::response::{OpType, Operation};

pub struct Op<'repo> {
    ops: Vec<Operation>,
    repo: &'repo Repository,
}

impl<'repo> Op<'repo> {
    pub fn init(
        ops: Vec<Operation>,
        repo: &'repo Repository,
    ) -> Op<'repo> {
        Op { ops, repo }
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
        self.repo.index().unwrap();
    }

    fn commit(&self) {}

    fn new_branch(&self) {}
}
