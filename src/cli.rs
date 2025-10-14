use clap::{Parser, Subcommand};

use crate::config::Config;

#[derive(Debug, Parser)]
#[command(name = "gai")]
#[command(about = "gai cli", long_about = None)]
pub struct Cli {
    /// send request on launch
    #[arg(long)]
    pub auto_request: bool,

    /// skip splash screen
    #[arg(long)]
    pub skip_splash: bool,

    /// include untracked files
    #[arg(long)]
    pub include_untracked: bool,

    /// apply changes as hunks
    #[arg(long)]
    pub stage_hunks: bool,

    /// path to API key file
    #[arg(long, value_name = "file")]
    pub api_key_file: Option<String>,

    /// include file tree in request
    #[arg(long)]
    pub include_file_tree: bool,

    /// files to truncate
    #[arg(long, value_name = "file")]
    pub truncate_file: Vec<String>,

    /// capitalize commit prefix
    #[arg(long)]
    pub capitalize_prefix: bool,

    /// include scope in commits
    #[arg(long)]
    pub include_scope: bool,

    /// custom system prompt
    #[arg(long, value_name = "prompt")]
    pub system_prompt: Option<String>,

    /// use conventional commits
    #[arg(long)]
    pub include_convention: bool,

    /// group related files
    #[arg(long)]
    pub group_related_files: bool,

    /// don't split files across commits
    #[arg(long)]
    pub no_file_splitting: bool,

    /// separate commits by purpose
    #[arg(long)]
    pub separate_by_purpose: bool,

    /// verbose commit descriptions
    #[arg(long)]
    pub verbose_descriptions: bool,

    /// exclude file extension in scope
    #[arg(long)]
    pub exclude_extension_in_scope: bool,

    /// allow empty scope
    #[arg(long)]
    pub allow_empty_scope: bool,

    /// enable OpenAI
    #[arg(long)]
    pub enable_openai: bool,

    /// OpenAI model name
    #[arg(long, value_name = "model")]
    pub openai_model: Option<String>,

    /// OpenAI max tokens
    #[arg(long, value_name = "num")]
    pub openai_max_tokens: Option<u64>,

    /// enable Gemini
    #[arg(long)]
    pub enable_gemini: bool,

    /// Gemini model name
    #[arg(long, value_name = "model")]
    pub gemini_model: Option<String>,

    /// Gemini max tokens
    #[arg(long, value_name = "num")]
    pub gemini_max_tokens: Option<u64>,

    /// enable Claude
    #[arg(long)]
    pub enable_claude: bool,

    /// Claude model name
    #[arg(long, value_name = "model")]
    pub claude_model: Option<String>,

    /// Claude max tokens
    #[arg(long, value_name = "num")]
    pub claude_max_tokens: Option<u64>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// run as tui
    Tui,
}

impl Cli {
    pub fn parse_args(&self, config: &mut Config) {
        if self.auto_request {
            config.auto_request = true;
        }
        if self.skip_splash {
            config.skip_splash = true;
        }
        if self.include_untracked {
            config.include_untracked = true;
        }
        if self.stage_hunks {
            config.stage_hunks = true;
        }
        if let Some(v) = &self.api_key_file {
            config.api_key_file = v.to_owned();
        }
        if self.include_file_tree {
            config.include_file_tree = true;
        }
        if !self.truncate_file.is_empty() {
            config.files_to_truncate = self.truncate_file.to_owned();
        }

        if self.capitalize_prefix {
            config.ai.capitalize_prefix = true;
        }
        if self.include_scope {
            config.ai.include_scope = true;
        }
        if let Some(v) = &self.system_prompt {
            config.ai.system_prompt = v.to_owned();
        }
        if self.include_convention {
            config.ai.include_convention = true;
        }

        if self.group_related_files {
            config.ai.rules.group_related_files = true;
        }
        if self.no_file_splitting {
            config.ai.rules.no_file_splitting = true;
        }
        if self.separate_by_purpose {
            config.ai.rules.separate_by_purpose = true;
        }
        if self.verbose_descriptions {
            config.ai.rules.verbose_descriptions = true;
        }
        if self.exclude_extension_in_scope {
            config.ai.rules.exclude_extension_in_scope = true;
        }
        if self.allow_empty_scope {
            config.ai.rules.allow_empty_scope = true;
        }

        if self.enable_openai {
            config.ai.openai.enable = true;
        }
        if let Some(v) = &self.openai_model {
            config.ai.openai.model_name = v.to_owned();
        }
        if let Some(v) = self.openai_max_tokens {
            config.ai.openai.max_tokens = v;
        }

        if self.enable_gemini {
            config.ai.gemini.enable = true;
        }
        if let Some(v) = &self.gemini_model {
            config.ai.gemini.model_name = v.to_owned();
        }
        if let Some(v) = self.gemini_max_tokens {
            config.ai.gemini.max_tokens = v;
        }

        if self.enable_claude {
            config.ai.claude.enable = true;
        }
        if let Some(v) = &self.claude_model {
            config.ai.claude.model_name = v.to_owned();
        }
        if let Some(v) = self.claude_max_tokens {
            config.ai.claude.max_tokens = v;
        }
    }
}
