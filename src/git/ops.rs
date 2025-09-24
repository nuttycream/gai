use crate::response::{OpType, Operation};

pub struct Op {
    ops: Vec<Operation>,
}

impl Op {
    pub fn init(ops: Vec<Operation>) -> Self {
        Self { ops }
    }

    pub fn apply_ops(&self) {
        for op in &self.ops {
            match op.op_type {
                OpType::AddFile => self.add(),
                OpType::StageFile => self.stage(),
                OpType::CommitChanges => self.commit(),
                OpType::NewBranch => self.new_branch(),
            }
        }
    }

    fn add(&self) {}

    fn stage(&self) {}

    fn commit(&self) {}

    fn new_branch(&self) {}
}
