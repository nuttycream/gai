use anyhow::{Result, anyhow};
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
use schemars::generate::SchemaSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{
    ai::response::ResponseSchema,
    auth::get_token,
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
    Gai,
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
                Provider::Gai => providers.insert(
                    provider,
                    ProviderConfig::new(GEMINI_DEFAULT),
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
    ) -> Result<ResponseSchema> {
        match self {
            Provider::Gai => {
                // atm rig-core doesn't seem to let us build our own client
                // realistically, we don't need a lot of it
                // since we can just create our own schema per provider
                // for gemini for example, the structured output schema
                // doesn't like additionalfields so we had to get rid of
                // deny_unknown_fields.
                // but openai requires this
                //
                // also rig-core was using a tool call to have the LLM
                // create its own structured output based on their own (provider) specs
                // it would be relatively flimsly and fail for us since we're targeting
                // the cheaper models which may not generate a proper structure
                // ideally we can restrict this with our own schema
                // but whether or not we generate it with schemars
                // is going to be up to decide later

                let generator = SchemaSettings::draft2020_12()
                    .with(|s| {
                        s.meta_schema = None;
                        s.inline_subschemas = true;
                    })
                    .into_generator();

                let schema = generator
                    .into_root_schema_for::<ResponseSchema>();

                let schema_value = serde_json::to_value(&schema)?;

                let content_text = format!("{}\n\n{}", prompt, diffs);

                let request_body = serde_json::json!({
                    "contents": [{
                        "parts": [{
                            "text": content_text
                        }]
                    }],
                    "generationConfig": {
                        "responseMimeType": "application/json",
                        "responseSchema": schema_value,
                        "maxOutputTokens": max_tokens
                    }
                });

                let auth_token = get_token()?;

                let endpoint = "https://cli.gai.fyi/generate";

                let client = reqwest::Client::new();
                let response = client
                    .post(endpoint)
                    .header(
                        "Authorization",
                        format!("Bearer {}", auth_token),
                    )
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| {
                        anyhow!("failed to send request: {}", e)
                    })?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text =
                        response.text().await.unwrap_or_else(|_| {
                            "Unknown error".to_string()
                        });
                    return Err(anyhow!(
                        "request failed with status {}: {}",
                        status,
                        error_text
                    ));
                }

                let response_json: serde_json::Value =
                    response.json().await.map_err(|e| {
                        anyhow!(
                            "Failed to parse response JSON: {}",
                            e
                        )
                    })?;

                let generated_text = response_json
                    .get("candidates")
                    .and_then(|c| c.get(0))
                    .and_then(|c| c.get("content"))
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.get(0))
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| {
                        anyhow!(
                            "Invalid response format from Gemini API"
                        )
                    })?;

                let result: ResponseSchema = serde_json::from_str(
                    generated_text,
                )
                .map_err(|e| {
                    anyhow!(
                        "faield to parse JSON into vlaid schema: {}",
                        e
                    )
                })?;

                Ok(result)
            }
            Provider::OpenAI => {
                let client = openai::Client::from_env();

                let extractor = client
                    .extractor::<ResponseSchema>(model)
                    .max_tokens(max_tokens)
                    .preamble(prompt)
                    .build();

                Ok(extractor.extract(diffs).await?)
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

                Ok(extractor.extract(diffs).await?)
            }
            Provider::Claude => {
                let client = anthropic::Client::from_env();

                let extractor = client
                    .extractor::<ResponseSchema>(model)
                    .max_tokens(max_tokens)
                    .preamble(prompt)
                    .build();

                Ok(extractor.extract(diffs).await?)
            }
        }
    }
}
