use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{ai::provider::Provider, config::ProviderConfig};

/// response object along with any errors
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Response {
    pub errors: Vec<String>,
    pub response_schema: HashMap<Provider, ResponseSchema>,
}

/// response object that a provider will respond with
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
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

impl ResponseSchema {
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

pub async fn get_responses(
    diffs: &str,
    prompt: &str,
    providers: HashMap<Provider, ProviderConfig>,
) -> mpsc::Receiver<Response> {
    let (tx, rx) = mpsc::channel(providers.iter().len());

    for (provider, provider_cfg) in providers {
        if provider_cfg.enable {
            let tx = tx.clone();

            let mut response = Response::default();

            let provider_cfg = provider_cfg.clone();

            let diffs = diffs.to_owned();
            let prompt = prompt.to_owned();
            let model = provider_cfg.model.clone();
            let max_tokens = provider_cfg.max_tokens;

            tokio::spawn(async move {
                match provider
                    .extract(&prompt, &model, max_tokens, &diffs)
                    .await
                {
                    Ok(r) => {
                        response.response_schema.insert(provider, r);
                    }
                    Err(e) => {
                        response.errors.push(format!("{:#}", e))
                    }
                }

                let _ = tx.send(response).await;
            });
        }
    }

    drop(tx);
    rx
}

pub async fn get_response(
    diffs: &str,
    prompt: &str,
    provider: Provider,
    provider_cfg: ProviderConfig,
) -> Response {
    let mut resp = Response::default();

    match provider
        .extract(
            prompt,
            &provider_cfg.model,
            provider_cfg.max_tokens,
            diffs,
        )
        .await
    {
        Ok(r) => {
            resp.response_schema.insert(provider, r);
        }
        Err(e) => {
            resp.errors.push(format!("{:#}", e));
        }
    }

    resp
}
