use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::request::{InputData, RequestBuilder};

#[derive(Serialize, Deserialize)]
pub struct AiProvider {
    pub chatgpt: AiConfig,
    pub claude: AiConfig,

    pub prompt: String,
    pub git_message_convention: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct AiConfig {
    pub enable: bool,
    pub api_key_name: String,
    pub max_tokens: u32,
}

impl AiProvider {
    pub fn new() -> Self {
        let mut git_message_convention = HashMap::new();

        git_message_convention
            .insert("feat".to_owned(), "feat".to_owned());
        git_message_convention
            .insert("fix".to_owned(), "fix".to_owned());
        git_message_convention
            .insert("refactor".to_owned(), "refactor".to_owned());

        AiProvider {
            chatgpt: AiConfig::new("OPENAI"),
            claude: AiConfig::new("CLAUDE"),

            prompt: "You're a big shot engineer with high ambitions, \
                can you generate with your infinite wisdom, a spectacular \
                commit message for the following git diffs.".to_owned(),

            git_message_convention,

        }
    }

    pub fn build_request(&self, diffs: &[String]) -> RequestBuilder {
        let mut rb = RequestBuilder::new("gpt-5-nano", &self.prompt);
        for diff in diffs {
            let mut input = InputData::new();
            input.add_data(&diff).unwrap();
            rb.add_input(input).unwrap();
        }

        rb
    }
}

impl AiConfig {
    pub fn new(api_key_name: &str) -> Self {
        Self {
            enable: true,
            api_key_name: api_key_name.to_owned(),
            max_tokens: 100,
        }
    }
}
