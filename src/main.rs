pub mod ai;
pub mod auth;
pub mod cli;
pub mod config;
pub mod consts;
pub mod git;
pub mod graph;
pub mod print;
pub mod tui;

use anyhow::Result;
use clap::Parser;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use dotenv::dotenv;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    io::{Stdout, stdout},
    time::Duration,
};

use crate::{
    ai::{request::Request, response::get_response},
    auth::{auth_login, auth_status, clear_auth},
    cli::{Auth, Cli, Commands},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    print::{pretty_print_commits, pretty_print_status},
    tui::run_tui,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut cfg = config::Config::init()?;

    let args = Cli::parse();

    args.parse_args(&mut cfg);

    match args.command {
        Commands::Auth { ref auth } => {
            let bar = create_spinner_bar();
            run_auth(auth, bar).await?;
        }
        _ => {
            let mut gai = GaiGit::new(
                cfg.gai.only_staged,
                cfg.gai.stage_hunks,
                cfg.gai.commit_config.capitalize_prefix,
                cfg.gai.commit_config.include_scope,
            )?;

            gai.create_diffs(&cfg.ai.files_to_truncate)?;

            let mut stdout = stdout();
            pretty_print_status(&mut stdout, &gai)?;

            let bar = create_spinner_bar();
            let req = build_request(&cfg, &gai, &bar);

            match args.command {
                Commands::Commit { skip_confirmation } => {
                    run_commit(
                        stdout,
                        bar,
                        req,
                        cfg,
                        gai,
                        skip_confirmation,
                    )
                    .await?
                }
                Commands::Status { print_request } => {
                    if print_request {
                        println!("{}", req);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn build_request(
    cfg: &Config,
    gai: &GaiGit,
    bar: &ProgressBar,
) -> Request {
    bar.set_message("Building Request...");
    let mut req = Request::default();
    req.build_prompt(cfg, gai);
    req.build_diffs_string(gai.get_file_diffs_as_str());
    bar.finish();
    req
}

fn create_spinner_bar() -> ProgressBar {
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(80));
    bar.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⣼", "⣹", "⢻", "⠿", "⡟", "⣏", "⣧", "⣶"]),
    );
    bar
}

async fn run_auth(auth: &Auth, bar: ProgressBar) -> Result<()> {
    match auth {
        Auth::Login => auth_login()?,
        Auth::Status => auth_status(bar).await?,
        Auth::Logout => clear_auth()?,
    }

    Ok(())
}

async fn run_commit(
    mut stdout: Stdout,
    bar: ProgressBar,
    req: Request,
    cfg: Config,
    gai: GaiGit,
    skip_confirmation: bool,
) -> Result<()> {
    loop {
        let provider = cfg.ai.provider;
        let provider_cfg = cfg
            .ai
            .providers
            .get(&provider)
            .expect("somehow did not find provider config");

        bar.reset();
        bar.set_message(format!(
            "Awaiting response from {} using {}",
            cfg.ai.provider, provider_cfg.model
        ));

        let response =
            get_response(&req, provider, provider_cfg.to_owned())
                .await;

        let result = match response.result.clone() {
            Ok(r) => r,
            Err(e) => {
                bar.finish_with_message(
                    "Done! But Gai received an error from the provider:",
                );

                println!("{:#}", e);

                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Retry?")
                    .interact()
                    .unwrap()
                {
                    continue;
                } else {
                    break;
                }
            }
        };

        if result.commits.is_empty() {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("No commits found... retry?")
                .interact()
                .unwrap()
            {
                continue;
            } else {
                break;
            }
        }

        let finished_msg = format!(
            "Done! Received {} Commit{}",
            result.commits.len(),
            if result.commits.len() == 1 { "" } else { "s" }
        );

        bar.finish_with_message(finished_msg);

        pretty_print_commits(
            &mut stdout,
            &result.commits,
            &cfg,
            &gai,
        )?;

        let commits: Vec<GaiCommit> = result
            .commits
            .iter()
            .map(|resp_commit| {
                GaiCommit::from_response(
                    resp_commit,
                    cfg.gai.commit_config.capitalize_prefix,
                    cfg.gai.commit_config.include_scope,
                )
            })
            .collect();

        if skip_confirmation {
            println!("Skipping confirmation and applying commits...");
            gai.apply_commits(&commits);
            break;
        }

        let options = [
            "Apply All",
            "Edit Commit/s (Opens the TUI)",
            "Retry",
            "Exit",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an option:")
            .items(options)
            .default(0)
            .interact()
            .unwrap();

        if selection == 0 {
            println!("Applying Commits...");
            gai.apply_commits(&commits);
        } else if selection == 1 {
            let _ = run_tui(req, cfg, gai, Some(response)).await;
        } else if selection == 2 {
            println!("Retrying...");
            continue;
        } else if selection == 3 {
            println!("Exiting");
        }

        break;
    }

    Ok(())
}
