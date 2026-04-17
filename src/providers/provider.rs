use llmao::extract::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::{Display, EnumIter};

use super::{
    gai::GaiConfig, gemini::GeminiConfig, openai::OpenAIConfig,
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
    clap::ValueEnum,
)]
pub enum ProviderKind {
    OpenAI,
    Gemini,
    Claude,
    Gai,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProviderSettings {
    pub gai: GaiConfig,
    pub openai: OpenAIConfig,
    pub gemini: GeminiConfig,
}

#[derive(Debug)]
pub enum ProviderError {
    HttpError(ureq::Error),
    ParseError(serde_json::Error),
    NoContent,
    InvalidSchema,
    NotAuthenticated,
}

impl Display for ProviderError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ProviderError::HttpError(e) => {
                write!(f, "HTTP error: {}", e)
            }
            ProviderError::ParseError(e) => {
                write!(f, "Parse error: {}", e)
            }
            ProviderError::NoContent => {
                write!(f, "No content in response")
            }
            ProviderError::InvalidSchema => {
                write!(f, "Invalid schema")
            }
            ProviderError::NotAuthenticated => {
                write!(f, "Not authenticated")
            }
        }
    }
}

impl Error for ProviderError {
    fn kind(&self) -> ErrorKind {
        match self {
            ProviderError::NoContent => ErrorKind::NoData,
            ProviderError::ParseError(_) => {
                ErrorKind::DeserializationFailed
            }
            ProviderError::InvalidSchema => ErrorKind::BadSchema,
            _ => ErrorKind::NoData,
        }
    }
}

impl From<ureq::Error> for ProviderError {
    fn from(e: ureq::Error) -> Self {
        ProviderError::HttpError(e)
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(e: serde_json::Error) -> Self {
        ProviderError::ParseError(e)
    }
}

impl ProviderSettings {
    pub fn get_model(
        &self,
        provider: &ProviderKind,
    ) -> &str {
        match provider {
            ProviderKind::OpenAI => &self.openai.model,
            ProviderKind::Gemini => &self.gemini.model,
            ProviderKind::Claude => "not yet implemented",
            ProviderKind::Gai => &self.gai.model,
        }
    }
}
