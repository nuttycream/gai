use strum::{Display, EnumIter, EnumString, IntoStaticStr};

use crate::ai::response::Response;

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    Eq,
    PartialEq,
    EnumIter,
    EnumString,
    Display,
    IntoStaticStr,
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
