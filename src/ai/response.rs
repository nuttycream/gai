use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Response {
    pub commits: Vec<ResponseCommit>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ResponseCommit {
    // paths to apply commit to
    // ex. git add main.rs doubloon.rs
    pub files: Vec<String>,
    // hunk headers per file
    // when stage_hunks is enabled
    pub hunk_headers: Vec<String>,
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

    /// description compoennts
    pub header: String,
    pub body: String,
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
        serde_json::from_str(response).unwrap()
    }
}

impl ResponseCommit {
    /// only used for UI for now, likely
    /// need to refactored out
    pub fn get_commit_prefix(
        &self,
        capitalize_prefix: bool,
        include_scope: bool,
    ) -> String {
        let prefix = if capitalize_prefix {
            format!("{:?}", self.message.prefix)
        } else {
            format!("{:?}", self.message.prefix).to_lowercase()
        };

        let breaking = if self.message.breaking { "!" } else { "" };

        let scope = if include_scope {
            format!("({})", self.message.scope.to_lowercase())
        } else {
            "".to_owned()
        };

        format!("{}{}{}", prefix, breaking, scope)
    }
}
