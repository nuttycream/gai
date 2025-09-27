use std::collections::HashMap;

use rig::agent::Agent;
use serde::{Deserialize, Serialize};

use crate::request::RequestBuilder;

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
    pub max_tokens: u32,
}

impl Default for AI {
    fn default() -> Self {
        Self {
            prompt: "You are an expert at git operations.\
            Create git a logical list of git operations \
            based on diffs and structure."
                .to_owned(),

            rules: String::new(),

            openai: AiConfig::new("gpt-5-nano-2025-08-07"),
            claude: AiConfig::new("claude-3-5-haiku-latest"),
            gemini: AiConfig::new("gemini-2.5-flash-lite"),
        }
    }
}

impl AI {
    pub fn build_requests(&self, diffs: HashMap<String, String>) {
        if self.openai.enable {}

        if self.gemini.enable {}

        if self.claude.enable {}
    }
}

impl AiConfig {
    pub fn new(model_name: &str) -> Self {
        Self {
            enable: false,
            model_name: model_name.to_owned(),
            max_tokens: 1024,
        }
    }
}
