// https://github.com/Boshen/criterion2.rs/tree/main/src

use std::str::FromStr;

use bpaf::{OptionParser, Parser, construct, long, short};

#[derive(Debug, Clone)]
pub enum Commands {
    Commit(crate::commit::CommitArgs),
}

#[derive(Debug, Clone)]
pub enum ColorAlways {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub verbose: bool,
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
        construct!([commit])
    };

    construct!(Options {
        verbose,
        color,
        config,
        commands,
    })
    .to_options()
    .fallback_to_usage()
    .version(env!("CARGO_PKG_VERSION"))
}
