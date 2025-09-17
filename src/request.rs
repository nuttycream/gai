use std::error::Error;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RequestBuilder {
    pub model: String,

    /// max response tokens
    //pub max_tokens: u32,

    /// the proompt
    instructions: String,

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
            content_type: "input_text".to_string(),
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

    pub fn add_data(&mut self, text: &str) {
        self.content.push(Content::data(text));
    }
}

impl RequestBuilder {
    /// todo: pass cfg here
    pub fn new(model: &str, instructions: &str) -> Self {
        Self {
            model: model.to_owned(),
            //max_tokens: 1000,
            instructions: instructions.to_owned(),
            input: Default::default(),
        }
    }

    pub fn add_input(&mut self, input: InputData) {
        self.input.push(input);
    }
}
