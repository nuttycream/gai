use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Response {
    pub ops: Vec<Operation>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Operation {
    pub op_type: OpType,

    // paths to apply operation to
    // ex. git add main.rs doubloon.rs
    pub files: Vec<String>,

    pub message: CommitMessage,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct CommitMessage {
    // feat
    pub prefix: PrefixType,
    // (api)
    pub scope: String,
    // !
    pub breaking: bool,
    // desc
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum OpType {
    AddFile,
    StageFile,
    CommitChanges,

    // should we?
    NewBranch,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum PrefixType {
    Feat,
    Fix,
    Refactor,
    Style,
    Test,
    Docs,
    Build,
    CI,
    Ops,
    Chore,

    // for newbranch
    // the ai may hallucinate
    // and use these
    // on non-new branch creations
    // should we even have these clankers
    // create branches?
    Merge,
    Revert,
}

impl Response {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn build_ops(&mut self, response: &str) {
        self.ops = Vec::new();
    }
}
