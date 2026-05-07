use clap::Parser;

pub mod args;
pub mod cmd;
pub mod git;
pub mod print;
pub mod providers;
pub mod requests;
pub mod responses;
pub mod schema;
pub mod settings;
pub mod utils;

use crate::args::Commands::{Commit, Find, Rebase, Reword, Status};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = args::Cli::parse();

    match &args.command {
        Status(a) => cmd::status::run(a, &args.global)?,
        Commit(a) => cmd::commit::run(a, &args.global)?,
        Find(a) => cmd::find::run(a, &args.global)?,
        Rebase(a) => cmd::rebase::run(a, &args.global)?,
        Reword(a) => cmd::reword::run(a, &args.global)?,
    };

    Ok(())
}
