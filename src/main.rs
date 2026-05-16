pub mod auth;
pub mod commit;
pub mod find;
pub mod git;
pub mod opts;
pub mod print;
pub mod providers;
pub mod rebase;
pub mod requests;
pub mod responses;
pub mod reword;
pub mod schema;
pub mod settings;
pub mod status;
pub mod utils;

use crate::{
    opts::{Commands, cli},
    settings::load::load,
};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let opts = cli().run();

    let settings = load(opts.config)?;

    match opts.commands {
        Commands::Commit(a) => commit::run(&a, &settings),
    }
}
