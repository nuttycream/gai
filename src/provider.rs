use std::{collections::HashMap, error::Error};

use rig::{
    client::{CompletionClient, ProviderClient},
    extractor::Extractor,
    providers::{
        anthropic,
        gemini::{
            self,
            completion::gemini_api_types::{
                AdditionalParameters, GenerationConfig,
            },
        },
        openai::{self, responses_api::ResponsesCompletionModel},
    },
};
use serde::{Deserialize, Serialize};

use crate::response::Response;

#[derive(Serialize, Deserialize)]
pub struct AI {
    pub prompt: String,

    pub openai: AiConfig,
    pub gemini: AiConfig,
    pub claude: AiConfig,

    pub rules: String,
}

#[derive(Serialize, Deserialize)]
pub struct AiConfig {
    pub enable: bool,
    pub model_name: String,
    pub max_tokens: u64,
}

impl Default for AI {
    fn default() -> Self {
        Self {
            prompt: "You are an expert at git operations.\
            Create git a logical list of git commits \
            based on diffs and structure."
                .to_owned(),

            rules: "
                - Make sure commits are atomic focusing on smaller changes.
                - Make multiple file stages and commits, if necessary.
                - Make sure you cover all files in the diff
                - IMPORTANT: Each file should appear in ONLY ONE commit. Do not create overlapping commits.
                - IMPORTANT: Do NOT create a summary commit that includes files already committed individually.
                - Choose EITHER:
                a) Multiple small commits (one per logical change/component), OR
                b) One larger commit for related changes across multiple files
                But NEVER both for the same set of files.
                - For CommitMessage:
                - Set message with:
                - prefix: The appropriate type from the PrefixType enum
                - scope: The component name or \"\"
                - breaking: true if breaking change, false otherwise
                - message: ONLY the description, do NOT include prefix or scope in the message text
                ".to_owned(),

            openai: AiConfig::new("gpt-5-nano-2025-08-07"),
            claude: AiConfig::new("claude-3-5-haiku-latest"),
            gemini: AiConfig::new("gemini-2.5-flash-lite"),
        }
    }
}

pub struct Extractors {
    pub gemini: Option<
        Extractor<gemini::completion::CompletionModel, Response>,
    >,
    pub claude: Option<
        Extractor<anthropic::completion::CompletionModel, Response>,
    >,
    pub openai: Option<Extractor<ResponsesCompletionModel, Response>>,
}

impl AI {
    pub fn build_requests(
        &self,
    ) -> Result<Extractors, Box<dyn Error>> {
        let prompt =
            format!("{}\nRules:\n{}", self.prompt, self.rules);
        let gemini = if self.gemini.enable {
            let client = gemini::Client::from_env();
            let gen_cfg = GenerationConfig {
                max_output_tokens: Some(self.gemini.max_tokens),
                ..Default::default()
            };

            let cfg =
                AdditionalParameters::default().with_config(gen_cfg);

            Some(
                client
                    .extractor::<Response>(&self.gemini.model_name)
                    .preamble(&prompt)
                    .additional_params(serde_json::to_value(cfg)?)
                    .build(),
            )
        } else {
            None
        };
        let openai = if self.openai.enable {
            let client = openai::Client::from_env();
            Some(
                client
                    .extractor::<Response>(&self.openai.model_name)
                    .preamble(&prompt)
                    .build(),
            )
        } else {
            None
        };
        let claude = if self.claude.enable {
            let client = anthropic::Client::from_env();
            Some(
                client
                    .extractor::<Response>(&self.claude.model_name)
                    .preamble(&prompt)
                    .build(),
            )
        } else {
            None
        };
        let extractors = Extractors {
            gemini,
            claude,
            openai,
        };
        Ok(extractors)
    }
}

impl AiConfig {
    pub fn new(model_name: &str) -> Self {
        Self {
            enable: false,
            model_name: model_name.to_owned(),
            max_tokens: 5000,
        }
    }
}
