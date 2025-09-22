use std::collections::HashMap;

use schemars::schema_for;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::response::Response;

#[derive(Debug, Serialize)]
pub struct RequestBuilder {
    pub model: String,

    /// max response tokens
    //pub max_tokens: u32,

    /// prompt
    pub input: Vec<InputData>,

    pub text: String,
}

#[derive(Default, Debug, Serialize)]
pub struct InputData {
    pub role: String,
    pub content: String,
}

impl RequestBuilder {
    pub fn new(model: &str) -> Self {
        let schema = schema_for!(Response);
        let text = serde_json::to_string_pretty(&schema).unwrap();
        //println!("{}", text);

        Self {
            model: model.to_owned(),
            //max_tokens: 1000,
            input: Vec::new(),

            text,
        }
    }

    pub fn add_input(&mut self, role: &str, content: &str) {
        self.input.push(InputData {
            role: role.to_owned(),
            content: content.to_owned(),
        });
    }

    /// for diff data,
    /// mapped with path as the key
    /// value as the changes
    pub fn add_diffs(&mut self, diffs: HashMap<String, String>) {
        for diff in diffs {}
    }
}
