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
pub mod state;
pub mod utils;

use crate::args::Commands::{
    Auth, Commit, Find, Log, Rebase, Reword, Status,
};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = args::Cli::parse();

    match &args.command {
        Auth(a) => cmd::auth::run(&a.auth)?,
        Status(a) => cmd::status::run(a, &args.global)?,
        Commit(a) => cmd::commit::run(a, &args.global)?,
        Log(a) => cmd::log::run(a, &args.global)?,
        Find(a) => cmd::find::run(a, &args.global)?,
        Rebase(a) => cmd::rebase::run(a, &args.global)?,
        Reword(a) => cmd::reword::run(a, &args.global)?,
    };

    Ok(())
}
