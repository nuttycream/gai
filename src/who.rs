use std::path::PathBuf;

use bpaf::{Parser, construct, short};

use crate::{opts::Commands, settings::Settings};

const WHO_DESC: &str = "\
LLM powered git blame. Collects git blame information across a file or
directory, groups it by author and commit, and sends it to the configured provider to 
answer a natural language question about the history.";

#[derive(Debug, Clone)]
pub struct WhoArgs {
    pub path: PathBuf,
    pub query: Option<String>,
}

pub fn who() -> impl Parser<Commands> {
    let path = short('p')
        .long("path")
        .help("File or directory to collect git blame from")
        .argument::<PathBuf>("PATH")
        .fallback(PathBuf::from("."));

    let query = short('q')
        .long("query")
        .help("Natural language question about the specified paths history.")
        .argument::<String>("QUERY")
        .optional();

    construct!(WhoArgs { path, query })
        .to_options()
        .descr(WHO_DESC)
        .command("who")
        .help("LLM powered git blame about a file or directory")
        .map(Commands::Who)
}

pub fn run(
    args: &WhoArgs,
    settings: &Settings,
) -> anyhow::Result<()> {
    todo!()
}
