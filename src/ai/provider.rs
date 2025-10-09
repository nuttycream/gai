use anyhow::Result;
use rig::{
    client::{CompletionClient, ProviderClient},
    extractor::ExtractionError,
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
use tokio::sync::mpsc;

use crate::{
    ai::response::Response,
    consts::{DEFAULT_RULES, DEFAULT_SYS_PROMPT},
    utils::build_prompt,
};

#[derive(Serialize, Deserialize)]
pub struct AI {
    pub capitalize_prefix: bool,
    pub include_scope: bool,
    pub system_prompt: String,

    /// conventionalcommits.md
    pub include_convention: bool,

    pub openai: AiConfig,
    pub gemini: AiConfig,
    pub claude: AiConfig,

    // do we want to expose this to the user?
    // maybe have 'predefined' options
    // for the rules, that they can toggle
    pub rules: String,
}

#[derive(Serialize, Deserialize)]
pub struct AiConfig {
    pub enable: bool,
    pub model_name: String,
    pub max_tokens: u64,
}

impl AiConfig {
    pub fn new(model_name: &str) -> Self {
        Self {
            enable: false,
            model_name: model_name.to_owned(),
            max_tokens: 5000,
        }
    }
}

impl Default for AI {
    fn default() -> Self {
        Self {
            system_prompt: DEFAULT_SYS_PROMPT.to_owned(),
            rules: DEFAULT_RULES.to_owned(),
            openai: AiConfig::new("gpt-5-nano-2025-08-07"),
            claude: AiConfig::new("claude-3-5-haiku-latest"),
            gemini: AiConfig::new("gemini-2.5-flash-lite"),

            include_convention: true,
            capitalize_prefix: false,
            include_scope: true,
        }
    }
}

impl AI {
    /// return receiver with:
    /// * `String` - the ai provider name
    /// * `Result<Response, String>` - The provider's response `Vec<Commit>` or
    ///   string based error message
    pub async fn get_responses(
        &self,
        diffs: &str,
        use_hunk: bool,
    ) -> Result<mpsc::Receiver<(String, Result<Response, String>)>>
    {
        let prompt = build_prompt(
            self.include_convention,
            &self.system_prompt,
            &self.rules,
            use_hunk,
        );

        let (tx, rx) = mpsc::channel(3);

        // according to examples and online refs
        // tokio::spawn needs a static lifetime
        // present to own its stuff, which means we
        // have to clone AND cant use self here
        // shouldnt be too expensive, but meh
        // maybe give the futures crate a look?

        if self.gemini.enable {
            let tx = tx.clone();
            let prompt = prompt.clone();
            let diffs = diffs.to_string();
            let model_name = self.gemini.model_name.clone();
            let max_tokens = self.gemini.max_tokens;

            tokio::spawn(async move {
                // println!("sending req to gemini");
                let provider = format!("Gemini({})", model_name);
                let result = try_gemini(
                    &prompt,
                    &model_name,
                    max_tokens,
                    &diffs,
                )
                .await
                .map_err(|e| format!("{:#}", e));

                let _ = tx.send((provider, result)).await;
            });
        }

        if self.openai.enable {
            let tx = tx.clone();
            let prompt = prompt.clone();
            let diffs = diffs.to_string();
            let model_name = self.openai.model_name.clone();

            tokio::spawn(async move {
                //println!("sending req to openai");
                let provider = format!("OpenAI({})", model_name);
                let resp = try_openai(&prompt, &model_name, &diffs)
                    .await
                    .map_err(|e| format!("{:#}", e));

                let _ = tx.send((provider, resp)).await;
            });
        }

        if self.claude.enable {
            let tx = tx.clone();
            let prompt = prompt.clone();
            let diffs = diffs.to_string();
            let model_name = self.claude.model_name.clone();

            tokio::spawn(async move {
                //println!("sending req to claude");
                let provider = format!("Claude({})", model_name);
                let resp = try_claude(&prompt, &model_name, &diffs)
                    .await
                    .map_err(|e| format!("{:#}", e));

                let _ = tx.send((provider, resp)).await;
            });
        }

        drop(tx);

        Ok(rx)
    }
}

// todo: ideally gemini wouldnt
// be the only model to accept max_tokens
// but its the fastest model atm, so after testing
// make sure you do this bud
async fn try_gemini(
    prompt: &str,
    model_name: &str,
    max_tokens: u64,
    diffs: &str,
) -> Result<Response, ExtractionError> {
    let client = gemini::Client::from_env();
    let gen_cfg = GenerationConfig {
        max_output_tokens: Some(max_tokens),
        ..Default::default()
    };

    let cfg = AdditionalParameters::default().with_config(gen_cfg);

    let extractor = client
        .extractor::<Response>(model_name)
        .preamble(prompt)
        .additional_params(serde_json::to_value(cfg)?)
        .build();

    extractor.extract(diffs).await
}

async fn try_openai(
    prompt: &str,
    model_name: &str,
    diffs: &str,
) -> Result<Response, ExtractionError> {
    let client = openai::Client::from_env();

    let extractor = client
        .extractor::<Response>(model_name)
        .preamble(prompt)
        .build();

    extractor.extract(diffs).await
}

async fn try_claude(
    prompt: &str,
    model_name: &str,
    diffs: &str,
) -> Result<Response, ExtractionError> {
    let client = anthropic::Client::from_env();

    let extractor = client
        .extractor::<Response>(model_name)
        .preamble(prompt)
        .build();

    extractor.extract(diffs).await
}
