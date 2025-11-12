use anyhow::Result;
use clap::{
    Parser, Subcommand,
    builder::styling::{self, AnsiColor},
};

use crate::{ai::provider::Provider, config::Config};

pub const STYLING: styling::Styles = clap::builder::Styles::styled()
    .header(AnsiColor::White.on_default().bold())
    .usage(AnsiColor::BrightBlue.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Magenta.on_default())
    .error(AnsiColor::Red.on_default().bold())
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Yellow.on_default());

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, styles = STYLING)]
pub struct Args {
    /// Print with compact outputs (no pretty trees)
    #[arg(short = 'c', long)]
    pub compact: bool,

    /// Show the TUI
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// Override the current provider
    #[arg(short = 'p', long)]
    pub provider: Option<Provider>,

    /// Provide an additional 'hint' to the LLM
    #[arg(short = 'H', long)]
    pub hint: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Authenticate with GitHub OAuth to use the Gai provider
    Auth {
        #[command(subcommand)]
        auth: Auth,
    },

    /// Prints gai repository status
    Status {
        /// Prints the verbose status which includes the
        /// request prompt and request diffs
        #[arg(short = 'v', long)]
        verbose: bool,
    },

    /// Create commits from the diffs in the working tree
    Commit {
        /// Skips the confirmation prompt and applies
        /// the commits
        #[arg(short = 'y', long)]
        skip_confirmation: bool,

        /// Only generate for currently
        /// staged files/hunks
        #[arg(short = 's', long)]
        staged: bool,

        /// Stage as hunks
        #[arg(short = 'H', long)]
        hunks: bool,

        /// Stage as files
        #[arg(short = 'f', long)]
        files: bool,

        /// Override config option for this command
        #[arg(short = 'c', long, value_name = "KEY=VALUE")]
        config: Vec<String>,
    },
    /* todo: implement, see feature tracking
    /// Rebase commits
    Rebase {},
    /// Find a specific commit
    Find
    /// Initiate interactive bisect
    Bisect
    */
}

#[derive(Debug, Subcommand)]
pub enum Auth {
    /// Login using GitHub OAuth
    Login,

    /// Get the status of the logged-in user
    /// including requests made and when the count
    /// resets
    Status,

    /// Logout/clear the stored user token
    Logout,
}

impl Args {
    pub fn parse_flags(&self, config: &mut Config) -> Result<()> {
        if let Some(provider) = self.provider {
            config.ai.provider = provider;
        }

        config.ai.hint = self.hint.to_owned();

        // good lord...
        if let Commands::Commit {
            staged,
            hunks,
            files,
            ..
        } = self.command
        {
            if staged {
                config.gai.only_staged = true;
            }
            if hunks {
                config.gai.stage_hunks = true;
            }
            if files {
                config.gai.stage_hunks = false;
            }
        }

        Ok(())
    }
}
