use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub ai_config: AIConfig,
    pub ignore_config: IgnoreConfig,
}
#[derive(Default, Serialize, Deserialize)]
pub struct AIConfig {}

#[derive(Default, Serialize, Deserialize)]
pub struct IgnoreConfig {
    /// files that gai will ignore
    /// this is separate from .gitignore
    /// you can use this to add specific files
    /// that are not really relevant to send to the AI provider
    /// such as a Cargo.lock or package-lock.json file
    /// which may take up valuable token space
    pub files_to_ignore: Vec<String>,
}

impl Config {
    pub fn init() -> Self {
        Config {
            ai_config: AIConfig::default(),
            ignore_config: IgnoreConfig::default(),
        }
    }
}
