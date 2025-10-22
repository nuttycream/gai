use std::collections::HashMap;

use rig::providers::openai;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::config::ProviderConfig;

use super::response::Response;

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

    pub fn new() -> HashMap<Provider, ProviderConfig> {
        let mut providers = HashMap::new();
        for provider in Provider::iter() {
            match provider {
                Provider::OpenAI => providers.insert(
                    provider,
                    ProviderConfig::new("gpt-5-nano"),
                ),
                Provider::Gemini => providers.insert(
                    provider,
                    ProviderConfig::new("gemini-2.5-flash-lite"),
                ),
                Provider::Claude => providers.insert(
                    provider,
                    ProviderConfig::new("claude-3-5-haiku"),
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
