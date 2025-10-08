use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct Response {
    pub commits: Vec<GaiCommit>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct GaiCommit {
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
        serde_json::from_str(response).unwrap()
    }
}

impl GaiCommit {
    pub fn get_commit_message(&self, cfg: &Config) -> String {
        let prefix = if cfg.ai.capitalize_prefix {
            format!("{:?}", self.message.prefix)
        } else {
            format!("{:?}", self.message.prefix).to_lowercase()
        };

        let breaking = if self.message.breaking { "!" } else { "" };
        let scope = if cfg.ai.include_scope {
            // gonna set it to lowercase PERMA
            // sometimes the AI responds with a scope
            // that includes the file extension and is capitalized
            // like (Respfileonse.rs) which looks ridiculous imo
            // the only way i can think of is to make it a rule to not include
            // extension names
            format!("({})", self.message.scope.to_lowercase())
        } else {
            "".to_owned()
        };

        format!(
            "{}{}{}: {}",
            prefix, scope, breaking, self.message.message
        )
    }

    pub fn get_commit_prefix(&self, cfg: &Config) -> String {
        let prefix = if cfg.ai.capitalize_prefix {
            format!("{:?}", self.message.prefix)
        } else {
            format!("{:?}", self.message.prefix).to_lowercase()
        };

        let breaking = if self.message.breaking { "!" } else { "" };
        let scope = if cfg.ai.include_scope {
            format!("({})", self.message.scope.to_lowercase())
        } else {
            "".to_owned()
        };

        format!("{}{}{}", prefix, breaking, scope)
    }
}
