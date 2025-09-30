use anyhow::Result;
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
    pub capitalize_prefix: bool,
    pub include_scope: bool,
    pub prompt: String,

    pub openai: AiConfig,
    pub gemini: AiConfig,
    pub claude: AiConfig,

    // do we want to expose this to the user?
    // maybe have 'predefined' options
    // for the rules, that they can toggle
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
                - GROUP related files into LOGICAL commits based on the type of change
                - Examples of files that should be grouped together:
                * Multiple files implementing the same feature
                * Files modified for the same bug fix
                * Related configuration and code changes
                * Test files with the code they test
                - Each file should appear in ONLY ONE commit
                - Create multiple commits when changes serve different purposes
                - For CommitMessages:
                * prefix: The appropriate type from the PrefixType enum
                * scope: The component name or \"\", DO NOT include the file extension please!
                * breaking: true if breaking change, false otherwise
                * message: ONLY the description, do NOT include prefix or scope in the message text
                ".to_owned(),


            openai: AiConfig::new("gpt-5-nano-2025-08-07"),
            claude: AiConfig::new("claude-3-5-haiku-latest"),
            gemini: AiConfig::new("gemini-2.5-flash-lite"),

            capitalize_prefix: false,
            include_scope: true,
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
    pub fn build_requests(&self) -> Result<Extractors> {
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
