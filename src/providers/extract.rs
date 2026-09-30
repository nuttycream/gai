use llmao::extract::Extract;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::requests::Request;

use super::{
    gemini::GeminiProvider,
    openai::OpenAIProvider,
    provider::{ProviderError, ProviderKind},
};

pub fn extract_from_provider<T>(
    provider: &ProviderKind,
    request: Request,
    schema: Value,
) -> Result<T, ProviderError>
where
    T: DeserializeOwned,
{
    let prompt = request
        .system
        .to_owned();
    let content = request.get_content_as_str();

    match provider {
        ProviderKind::OpenAI => OpenAIProvider::new()
            .schema(schema)
            .extract(prompt, content),
        ProviderKind::Gemini => GeminiProvider::new()
            .schema(schema)
            .extract(prompt, content),

        _ => unreachable!(),
    }
}
