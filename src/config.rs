use std::{error::Error, fs, io::ErrorKind};

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    /// send out the request upon launching
    /// gai
    pub auto_request: bool,
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
    pub fn init(path: &str) -> Result<Self, Box<dyn Error>> {
        let cfg_str = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let def = Config::default();
                let def_toml = toml::to_string_pretty(&def)?;
                fs::write(path, &def_toml)?;
                def_toml
            }
            Err(e) => return Err(e.into()),
        };

        let cfg: Config = toml::from_str(&cfg_str)?;
        Ok(cfg)
    }
}
