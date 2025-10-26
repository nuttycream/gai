use clap::{Parser, Subcommand};

use crate::{ai::provider::Provider, config::Config};

#[derive(Debug, Parser)]
#[command(name = "gai")]
#[command(version)]
#[command(about, long_about = None)]
#[command(override_usage = "\n  gai [OPTIONS] [COMMAND]")]
pub struct Cli {
    /// include untracked files
    #[arg(short = 'u', long)]
    pub include_untracked: bool,

    /// apply changes as hunks
    #[arg(short = 'H', long)]
    pub stage_hunks: bool,

    /// path to API key file
    #[arg(short = 'k', long, value_name = "file")]
    pub api_key_file: Option<String>,

    /// include file tree in request
    #[arg(short = 't', long)]
    pub include_file_tree: bool,

    /// files to truncate
    #[arg(short = 'T', long, value_name = "file")]
    pub truncate_file: Vec<String>,

    /// capitalize commit prefix
    #[arg(short = 'c', long)]
    pub capitalize_prefix: bool,

    /// include scope in commits
    #[arg(short = 's', long)]
    pub include_scope: bool,

    /// custom system prompt
    #[arg(short = 'p', long, value_name = "prompt")]
    pub system_prompt: Option<String>,

    /// use conventional commits
    #[arg(short = 'C', long)]
    pub include_convention: bool,

    /// group related files
    #[arg(short = 'g', long)]
    pub group_related_files: bool,

    /// don't split files across hunks
    #[arg(short = 'S', long)]
    pub no_file_splitting: bool,

    /// separate commits by purpose
    #[arg(short = 'P', long)]
    pub separate_by_purpose: bool,

    /// verbose commit descriptions
    #[arg(short = 'v', long)]
    pub verbose_descriptions: bool,

    /// exclude file extension in scope
    #[arg(short = 'e', long)]
    pub exclude_extension_in_scope: bool,

    /// allow empty scope
    #[arg(short = 'E', long)]
    pub allow_empty_scope: bool,

    /// max commit message header length
    #[arg(short = 'm', long, value_name = "u16")]
    pub max_header_length: Option<u16>,

    /// max commit message body length
    #[arg(short = 'M', long, value_name = "u16")]
    pub max_body_length: Option<u16>,

    /// Force use ChatGPT
    #[arg(long)]
    pub chatgpt: bool,

    /// Force use Gemini
    #[arg(long)]
    pub gemini: bool,

    /// Force use Claude
    #[arg(long)]
    pub claude: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Open Terminal User Interface
    Tui {
        /// send request on launch
        #[arg(long)]
        auto_request: bool,

        /// skip splash screen
        #[arg(long)]
        skip_splash: bool,
    },

    /// Create commits
    Commit {
        /// Skips the confirmation prompt and applies
        /// the commits
        #[arg(short = 'y', long)]
        skip_confirmation: bool,
    },

    /// Rebase commits
    Rebase {},

    /// Find a specific commit
    Find {
        /// Insert range for commits to search from
        #[arg(long)]
        range: Option<u32>,

        /// Prompt to search for
        #[arg(long)]
        prompt: String,
    },

    /// Initiate interactive bisect
    Bisect {},
}

impl Cli {
    pub fn parse_args(&self, config: &mut Config) {
        config.ai.provider = if self.gemini {
            Provider::Gemini
        } else if self.chatgpt {
            Provider::OpenAI
        } else if self.claude {
            Provider::Claude
        } else {
            config.ai.provider
        }
    }
}
