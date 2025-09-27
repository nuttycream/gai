use std::{error::Error, fs, io::ErrorKind};

use serde::{Deserialize, Serialize};

use crate::provider::AI;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    /// send out the request
    /// upon launching gai
    pub auto_request: bool,

    pub skip_splash: bool,

    pub ai: AI,
    pub api_key_file: String,

    /// files that gai will ignore
    /// this is separate from .gitignore
    /// you can use this to add specific files
    /// that are not really relevant to send to the AI provider
    /// such as a Cargo.lock or package-lock.json file
    /// which may take up valuable token space
    pub files_to_ignore: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            auto_request: false,
            skip_splash: true,
            ai: AI::default(),
            api_key_file: "".to_owned(),
            files_to_ignore: vec![
                "Cargo.lock".to_owned(),
                "package-lock.json".to_owned(),
            ],
        }
    }

    pub fn init(path: &str) -> Result<Self, Box<dyn Error>> {
        let cfg_str = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let def = Config::new();
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
