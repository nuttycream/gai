pub mod commit;
pub mod config;
pub mod find;
pub mod git;
pub mod opts;
pub mod print;
pub mod providers;
pub mod rebase;
pub mod requests;
pub mod responses;
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

    match opts.commands {
        Commands::Commit(a) => commit::run(&a, &load(opts.config)?),
        Commands::Rebase(a) => rebase::run(&a, &load(opts.config)?),
        Commands::Find(a) => find::run(&a, &load(opts.config)?),
        Commands::Status(a) => status::run(&a, &load(opts.config)?),
        Commands::Config(a) => config::run(&a, opts.config),
    }
}
