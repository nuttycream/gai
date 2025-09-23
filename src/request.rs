use std::collections::HashMap;

use schemars::generate::{Contract, SchemaSettings};
use serde::Serialize;
use serde_json::json;

use crate::response::Response;

#[derive(Debug, Serialize)]
pub struct RequestBuilder {
    pub model: String,

    /// max response tokens
    //pub max_tokens: u32,

    /// prompt
    pub input: Vec<InputData>,

    /// response schema
    pub text: serde_json::Value,
}

#[derive(Default, Debug, Serialize)]
pub struct InputData {
    pub role: String,
    pub content: String,
}

impl RequestBuilder {
    pub fn new(model: &str) -> Self {
        let generator = SchemaSettings::default()
            .with(|s| {
                s.meta_schema = None;
                s.inline_subschemas = true;
                s.untagged_enum_variant_titles = true;
                //s.contract = Contract::Serialize;
            })
            .into_generator();
        let schema = generator.into_root_schema_for::<Response>();

        //println!("{:#?}", schema.clone().to_value());

        let mut json = serde_json::to_value(&schema).unwrap();

        if let Some(obj) = json.as_object_mut() {
            obj.remove("title");
        }

        //println!("{}", serde_json::to_string_pretty(&json).unwrap());

        let text = json!({
            "format": {
                "type": "json_schema",
                "name": "git_ops",
                "schema": json,
                "strict": true,
            },
        });

        //println!("{}", text);

        Self {
            model: model.to_owned(),
            //max_tokens: 5000,
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
    /// these are inputdata with the role
    pub fn add_diffs(&mut self, diffs: HashMap<String, String>) {
        let mut content = String::from("diffs:\n");

        for (path, changes) in &diffs {
            content.push_str(&format!("file:{}\n", path));
            content.push_str("changes:\n");
            content.push_str(&changes);
        }

        let input_data = InputData {
            role: "user".to_owned(),
            content,
        };

        self.input.push(input_data);
    }
}
