use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Response {
    pub ops: Vec<Operation>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Operation {
    pub op_type: OpType,

    // paths to apply operation to
    // ex. git add main.rs doubloon.rs
    pub files: Option<Vec<String>>,

    pub message: Option<RespMessage>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct RespMessage {
    // feat
    pub prefix: PrefixType,
    // (api)
    pub scope: Option<String>,
    // !
    pub breaking: bool,
    // desc
    pub message: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum OpType {
    Add,
    Commit,
    NewBranch,
}

#[derive(Serialize, Deserialize, JsonSchema)]
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
