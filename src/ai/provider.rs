use std::collections::HashMap;

use rig::extractor::ExtractionError;
use rig::{
    client::{CompletionClient, ProviderClient},
    providers::{
        anthropic,
        gemini::{
            self,
            completion::gemini_api_types::{
                AdditionalParameters, GenerationConfig,
            },
        },
        openai,
    },
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{
    ai::response::ResponseSchema,
    config::ProviderConfig,
    consts::{CHATGPT_DEFAULT, CLAUDE_DEFAULT, GEMINI_DEFAULT},
};

#[derive(
    Clone,
    Copy,
    Debug,
    Hash,
    Eq,
    PartialEq,
    EnumIter,
    Display,
    Serialize,
    Deserialize,
)]
pub enum Provider {
    OpenAI,
    Gemini,
    Claude,
}

impl Provider {
    pub fn name(&self, model: &str) -> String {
        format!("{} ({})", self, model)
    }

    pub fn create_defaults() -> HashMap<Provider, ProviderConfig> {
        let mut providers = HashMap::new();
        for provider in Provider::iter() {
            match provider {
                Provider::OpenAI => providers.insert(
                    provider,
                    ProviderConfig::new(CHATGPT_DEFAULT),
                ),
                Provider::Gemini => providers.insert(
                    provider,
                    ProviderConfig::new(GEMINI_DEFAULT),
                ),
                Provider::Claude => providers.insert(
                    provider,
                    ProviderConfig::new(CLAUDE_DEFAULT),
                ),
            };
        }

        providers
    }

    pub async fn extract(
        &self,
        prompt: &str,
        model: &str,
        max_tokens: u64,
        diffs: &str,
    ) -> Result<ResponseSchema, ExtractionError> {
        match self {
            Provider::OpenAI => {
                let client = openai::Client::from_env();

                let extractor = client
                    .extractor::<ResponseSchema>(model)
                    .max_tokens(max_tokens)
                    .preamble(prompt)
                    .build();

                extractor.extract(diffs).await
            }
            Provider::Gemini => {
                let client = gemini::Client::from_env();
                let gen_cfg = GenerationConfig {
                    max_output_tokens: Some(max_tokens),
                    ..Default::default()
                };

                let cfg = AdditionalParameters::default()
                    .with_config(gen_cfg);

                let extractor = client
                    .extractor::<ResponseSchema>(model)
                    .preamble(prompt)
                    .additional_params(
                        serde_json::to_value(cfg).unwrap(),
                    )
                    .build();

                extractor.extract(diffs).await
            }
            Provider::Claude => {
                let client = anthropic::Client::from_env();

                let extractor = client
                    .extractor::<ResponseSchema>(model)
                    .max_tokens(max_tokens)
                    .preamble(prompt)
                    .build();

                extractor.extract(diffs).await
            }
        }
    }
}
