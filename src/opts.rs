// https://github.com/Boshen/criterion2.rs/tree/main/src

use std::str::FromStr;

use bpaf::{OptionParser, Parser, construct, long, short};

#[derive(Debug, Clone)]
pub enum Commands {
    Commit(crate::commit::CommitArgs),
    Rebase(crate::rebase::RebaseArgs),
    Find(crate::find::FindArgs),
    Status(crate::status::StatusArgs),
    Config(crate::config::ConfigSubCommands),
}

#[derive(Debug, Clone)]
pub enum ColorAlways {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone)]
pub struct Options {
    //FIXME: unused
    pub verbose: bool,
    //FIXME: unused
    pub quiet: bool,
    pub color: ColorAlways,
    pub config: Option<Vec<String>>,
    pub commands: Commands,
}

impl FromStr for ColorAlways {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "auto" => Self::Auto,
            "always" => Self::Always,
            "never" => Self::Never,
            _ => {
                return Err("expected auto|always|never");
            }
        })
    }
}

pub fn cli() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Show detailed information when running commands")
        .switch();

    let quiet = short('q')
        .long("quiet")
        .help(
            "Suppress all print statements, and confirmation prompts",
        )
        .switch();

    let color = long("color")
        .help("Allow color: auto, always, or never")
        .argument::<ColorAlways>("WHEN")
        .fallback(ColorAlways::Auto);

    let config = short('c')
        .long("config")
        .help("Override gai config value, separated by ','")
        .argument::<String>("KEY=VALUE")
        .map(|s| {
            s.split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .optional();

    let commands = {
        let commit = crate::commit::commit();
        let rebase = crate::rebase::rebase();
        let find = crate::find::find();
        let status = crate::status::status();
        let config = crate::config::config();
        construct!([commit, rebase, find, status, config])
    };

    construct!(Options {
        verbose,
        quiet,
        color,
        config,
        commands,
    })
    .to_options()
    .fallback_to_usage()
    .version(env!("CARGO_PKG_VERSION"))
}
