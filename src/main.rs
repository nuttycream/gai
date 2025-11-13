pub mod ai;
pub mod args;
pub mod auth;
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

use crate::{
    ai::{request::Request, response::get_response},
    args::{Args, Auth, Commands},
    auth::{auth_login, auth_status, clear_auth},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    print::{SpinDeez, pretty_print_commits, pretty_print_status},
    tui::run_tui,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut cfg = config::Config::init()?;

    let args = Args::parse();
    let spinner = SpinDeez::new()?;

    args.parse_flags(&mut cfg)?;

    match args.command {
        Commands::Auth { ref auth } => {
            run_auth(auth, &spinner).await?;
        }

        _ => {
            let mut gai = GaiGit::new(
                cfg.gai.only_staged,
                cfg.gai.stage_hunks,
                cfg.gai.commit_config.capitalize_prefix,
                cfg.gai.commit_config.include_scope,
            )?;

            gai.create_diffs(&cfg.ai.files_to_truncate)?;

            if args.interactive {
                let req = build_request(&cfg, &gai, &spinner);
                run_tui(req, cfg, gai, None).await?;
                return Ok(());
            }

            pretty_print_status(&gai)?;

            match args.command {
                Commands::Commit {
                    skip_confirmation,
                    config,
                    ..
                } => {
                    let cfg = match config {
                        Some(c) => cfg.override_cfg(&c)?,
                        None => cfg,
                    };

                    let req = build_request(&cfg, &gai, &spinner);

                    run_commit(
                        &spinner,
                        req,
                        cfg,
                        gai,
                        skip_confirmation,
                    )
                    .await?
                }
                Commands::Status { verbose } => {
                    if verbose {
                        let req = build_request(&cfg, &gai, &spinner);
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
    spinner: &SpinDeez,
) -> Request {
    spinner.start("Building Request...");
    let mut req = Request::default();
    req.build_prompt(cfg, gai);
    req.build_diffs_string(gai.get_file_diffs_as_str());
    spinner.stop(None);
    req
}

async fn run_auth(auth: &Auth, spinner: &SpinDeez) -> Result<()> {
    match auth {
        Auth::Login => auth_login()?,
        Auth::Status => auth_status(spinner).await?,
        Auth::Logout => clear_auth()?,
    }

    Ok(())
}

async fn run_commit(
    spinner: &SpinDeez,
    req: Request,
    cfg: Config,
    gai: GaiGit,
    skip_confirmation: bool,
) -> Result<()> {
    let provider = cfg.ai.provider;
    let provider_cfg = cfg
        .ai
        .providers
        .get(&provider)
        .expect("somehow did not find provider config");

    loop {
        spinner.start(&format!(
            "Awaiting response from {} using {}",
            cfg.ai.provider, provider_cfg.model
        ));

        let response =
            get_response(&req, provider, provider_cfg.to_owned())
                .await;

        let result = match response.result.clone() {
            Ok(r) => r,
            Err(e) => {
                spinner.stop(Some(
                    "Done! But Gai received an error from the provider:"
                ));

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

        spinner.stop(None);

        println!(
            "Done! Received {} Commit{}",
            result.commits.len(),
            if result.commits.len() == 1 { "" } else { "s" }
        );

        pretty_print_commits(&result.commits, &cfg, &gai)?;

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

        let options = ["Apply All", "Show in TUI", "Retry", "Exit"];

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
