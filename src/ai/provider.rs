use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::{
    ai::response::Response,
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
        max_tokens: u32,
        diffs: &str,
    ) -> Response {
        match self {
            Provider::OpenAI => todo!(),
            Provider::Gemini => todo!(),
            Provider::Claude => todo!(),
        }
    }
}
