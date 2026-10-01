use bpaf::{Parser, construct, pure};

use crate::{opts::Commands, settings::load::load};

#[derive(Debug, Clone)]
pub enum ConfigSubCommands {
    Init,
    Path,
    Show,
}

pub fn config() -> impl Parser<Commands> {
    let init = pure(ConfigSubCommands::Init)
        .to_options()
        .command("init")
        .help("Create a config file using defaults");

    let path = pure(ConfigSubCommands::Path)
        .to_options()
        .command("path")
        .help("Print the config file location");

    let show = pure(ConfigSubCommands::Show)
        .to_options()
        .command("show")
        .help("Print the current loaded config");

    construct!([init, path, show])
        .to_options()
        .command("config")
        .help("Manage gai config")
        .map(Commands::Config)
}

pub fn run(
    sub: &ConfigSubCommands,
    overrides: Option<Vec<String>>,
) -> anyhow::Result<()> {
    match sub {
        ConfigSubCommands::Init => {}
        ConfigSubCommands::Path => {}
        ConfigSubCommands::Show => {
            let settings = load(overrides)?;

            println!("{:#?}", settings);
        }
    }

    Ok(())
}
