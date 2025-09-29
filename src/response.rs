use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Response {
    pub commits: Vec<Commit>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Commit {
    // paths to apply commit to
    // ex. git add main.rs doubloon.rs
    pub files: Vec<String>,
    pub message: CommitMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
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
    pub fn new(response: &str) -> Self {
        return serde_json::from_str(response).unwrap();
    }
}
