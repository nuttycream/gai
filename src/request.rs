use std::error::Error;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RequestBody {
    pub model: String,

    /// max response tokens
    pub max_tokens: u32,

    /// prompt + data
    pub input: Vec<InputData>,
}

#[derive(Default, Debug, Serialize)]
pub struct InputData {
    pub role: String,
    pub content: Vec<Content>,
}

#[derive(Default, Debug, Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl Content {
    pub fn data(text: &str) -> Self {
        Self {
            content_type: "input_type".to_string(),
            text: text.to_owned(),
        }
    }
}

impl InputData {
    pub fn new() -> Self {
        Self {
            role: "user".to_owned(),
            content: Vec::new(),
        }
    }

    pub fn add_data(
        &mut self,
        text: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.content.push(Content::data(text));
        Ok(())
    }
}

impl RequestBody {
    /// todo: pass cfg here
    pub fn new() -> Self {
        Self {
            model: "gpt-5-nano-2025-08-07".to_owned(),
            max_tokens: 1000,
            input: Default::default(),
        }
    }

    pub fn add_input(
        &mut self,
        input: InputData,
    ) -> Result<(), Box<dyn Error>> {
        self.input.push(input);
        Ok(())
    }
}
