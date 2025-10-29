use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::ErrorKind};

use crate::ai::provider::Provider;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub ai: AiConfig,
    pub gai: GaiConfig,
    pub tui: TuiConfig,
}

impl Config {
    /// creates anew if it doesn't exist
    /// todo: this thing makes it whereever u call it
    /// fix IT please
    pub fn init(path: &str) -> Result<Self> {
        let cfg_str = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("No config.toml found. Creating anew.");
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
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GaiConfig {
    /// should we apply as hunks?
    pub stage_hunks: bool,
    pub commit_config: CommitConfig,
}

/// commit message specific settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitConfig {
    /// prefix will be capitalized like feat -> Feat
    pub capitalize_prefix: bool,

    /// the ai can respond with scopes
    /// instead of making it optional in the Schema
    /// (not all models support this)
    /// define it here before we commit
    pub include_scope: bool,

    /// use breaking symbol
    pub include_breaking: bool,

    /// breaking override defaults to "!"
    pub breaking_symbol: Option<char>,
    // todo make hashmap for customizable prefix types
    // todo allow user customizable format
}

/// tui specific settings
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TuiConfig {
    /// send out the request
    /// upon launching gai
    pub auto_request: bool,
    // todo impl keybinds
}

/// anything dealing with the LLM request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiConfig {
    /// Enabled provider
    pub provider: Provider,
    /// provider specific configuration
    pub providers: HashMap<Provider, ProviderConfig>,

    /// this is what tells the llm
    /// how to behave
    /// Defaults to NONE and will use
    /// the default_sys_prompt
    /// this is only for overriding
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
}

/// this is rules/constraints to send the ai
/// along with the prompt
#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub model: String,
    pub max_tokens: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Gemini,
            system_prompt: None,
            commit_convention: None,
            include_convention: true,
            include_file_tree: true,
            include_git_status: true,
            include_untracked: true,
            files_to_truncate: vec![],
            rules: RuleConfig::default(),
            providers: Provider::create_defaults(),
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

impl Default for CommitConfig {
    fn default() -> Self {
        Self {
            capitalize_prefix: true,
            include_scope: true,
            include_breaking: true,
            breaking_symbol: None,
        }
    }
}

impl ProviderConfig {
    pub fn new(model_name: &str) -> Self {
        Self {
            model: model_name.to_owned(),
            max_tokens: 5000,
        }
    }
}
