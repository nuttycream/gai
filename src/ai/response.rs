use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ai::{provider::Provider, request::Request},
    config::ProviderConfig,
};

/// response object along with any errors
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub result: Result<ResponseSchema, String>,
}

/// response object that a provider will respond with
#[derive(
    Clone, Debug, Default, Serialize, Deserialize, JsonSchema,
)]
#[schemars(deny_unknown_fields)]
pub struct ResponseSchema {
    pub commits: Vec<ResponseCommit>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ResponseCommit {
    // paths to apply commit to
    // ex. git add main.rs doubloon.rs
    pub files: Vec<String>,

    // hunk "ids" per file, more like
    // indices
    // when stage_hunks is enabled
    // ex: src/main.rs:0
    pub hunk_ids: Vec<String>,
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

impl ResponseCommit {
    /// only used for UI for now
    /// todo need to refactored out
    pub fn get_commit_prefix(
        &self,
        capitalize_prefix: bool,
        include_scope: bool,
    ) -> String {
        let prefix = if capitalize_prefix {
            format!("{:?}", self.message.prefix).to_uppercase()
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

pub async fn get_response(
    req: &Request,
    provider: Provider,
    provider_cfg: ProviderConfig,
) -> Response {
    let res = provider
        .extract(
            &req.prompt,
            &provider_cfg.model,
            provider_cfg.max_tokens,
            &req.diffs,
        )
        .await
        .map_err(|e| format!("{:#}", e));

    Response { result: res }
}
