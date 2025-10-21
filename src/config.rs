use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::ErrorKind};

use crate::consts::{COMMIT_CONVENTION, DEFAULT_SYS_PROMPT};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    gai: GaiConfig,
    tui: TuiConfig,
    ai: AiConfig,
}

impl Config {
    /// creates anew if it doesn't exist
    /// todo: this thing makes it whereever u call it
    /// fix IT please
    pub fn init(path: &str) -> Result<Self> {
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

/// gai git specific settings
#[derive(Debug, Serialize, Deserialize)]
pub struct GaiConfig {
    /// should we apply as hunks?
    pub stage_hunks: bool,
}

/// tui specific settings
#[derive(Debug, Serialize, Deserialize)]
pub struct TuiConfig {
    /// send out the request
    /// upon launching gai
    pub auto_request: bool,

    /// skip the wicked splash screen
    pub skip_splash: bool,
    // todo impl keybinds
}

/// anything dealing with the LLM request
#[derive(Debug, Serialize, Deserialize)]
pub struct AiConfig {
    /// this is what tells the llm
    /// how to behave
    pub system_prompt: Option<String>,

    /// commit convention v1 override
    pub commit_convention: Option<String>,

    /// include commit convention
    /// (note: this takes a lot of tokens)
    pub include_convention: bool,

    /// include git repo file tree in request
    pub include_file_tree: bool,

    /// include git status
    pub include_git_status: bool,

    /// should we send untracked files as well?
    pub include_untracked: bool,

    /// files that gai will be TRUNCATED
    /// you can use this to add specific files
    /// that are not really relevant to send to the AI provider
    /// such as a Cargo.lock or package-lock.json file
    /// which may take up valuable token space
    pub files_to_truncate: Vec<String>,

    /// ai response constraint/rules
    pub rules: RuleConfig,

    /// provider specific configuration
    pub providers: Providers,
}

/// this is rules/constraints to send the ai
/// along with the prompt
#[derive(Debug, Serialize, Deserialize)]
pub struct RuleConfig {
    /// group related files into logical commits based on the type of prefix
    pub group_related_files: bool,

    /// dont split single files, each file should be in ONE commit
    /// for hunk staging, this may be ignored imo, otherwise
    /// might have to keep this perma true
    pub no_file_splitting: bool,

    /// create SEPARATE commits when changes serve different purposes
    /// as in dont lump unrelated changes into one commit
    pub separate_by_purpose: bool,

    /// llm based verbosity
    pub verbose_descriptions: bool,

    /// file extensions in scope feat(git.rs) vs feat(git)
    pub exclude_extension_in_scope: bool,

    /// empty scope scope can be "" in the response
    pub allow_empty_scope: bool,

    // todo add hard validation
    /// max length of commit headers
    pub max_header_length: u16,

    // todo add hard validation
    /// max length of commit body
    pub max_body_length: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Providers {
    pub openai: ProviderConfig,
    pub gemini: ProviderConfig,
    pub claude: ProviderConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enable: bool,
    pub model: String,
    pub max_tokens: u32,
}

impl Default for GaiConfig {
    fn default() -> Self {
        Self { stage_hunks: false }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            auto_request: false,
            skip_splash: false,
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            system_prompt: Some(DEFAULT_SYS_PROMPT.to_owned()),
            commit_convention: Some(COMMIT_CONVENTION.to_owned()),
            include_convention: true,
            include_file_tree: true,
            include_git_status: true,
            include_untracked: true,
            files_to_truncate: vec![],
            rules: RuleConfig::default(),
            providers: Providers::default(),
        }
    }
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            group_related_files: true,
            no_file_splitting: true,
            separate_by_purpose: true,
            verbose_descriptions: true,
            exclude_extension_in_scope: true,
            allow_empty_scope: true,
            max_header_length: 52,
            max_body_length: 72,
        }
    }
}

impl Default for Providers {
    fn default() -> Self {
        Self {
            openai: ProviderConfig::new("gpt-5-nano"),
            gemini: ProviderConfig::new("claude-3-5-haiku"),
            claude: ProviderConfig::new("gemini-2.5-flash-lite"),
        }
    }
}

impl ProviderConfig {
    fn new(model_name: &str) -> Self {
        Self {
            enable: false,
            model: model_name.to_owned(),
            max_tokens: 5000,
        }
    }
}
