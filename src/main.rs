pub mod ai;
pub mod cli;
pub mod config;
pub mod consts;
pub mod git;
pub mod tui;
pub mod utils;

use anyhow::Result;
use clap::Parser;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use dotenv::dotenv;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::{
    ai::response::get_response,
    cli::{Cli, Commands},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    utils::{build_diffs_string, build_prompt},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut cfg = config::Config::init()?;

    let args = Cli::parse();

    args.parse_args(&mut cfg);

    let mut gai = GaiGit::new(
        cfg.gai.stage_hunks,
        cfg.gai.commit_config.capitalize_prefix,
        cfg.gai.commit_config.include_scope,
    )?;

    gai.create_diffs(&cfg.ai.files_to_truncate)?;
    match args.command {
        Commands::Tui { .. } => tui::run_tui(cfg, gai, None).await?,
        Commands::Commit { skip_confirmation } => {
            run_commit(cfg, gai, skip_confirmation).await?
        }
        Commands::Find { .. } => println!("Not yet implemented"),
        Commands::Rebase {} => println!("Not yet implemented"),
        Commands::Bisect {} => println!("Not yet implemented"),
    }

    Ok(())
}

async fn run_commit(
    cfg: Config,
    gai: GaiGit,
    skip_confirmation: bool,
) -> Result<()> {
    loop {
        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(80));
        bar.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&[
                    "⣼", "⣹", "⢻", "⠿", "⡟", "⣏", "⣧", "⣶",
                ]),
        );

        let provider = cfg.ai.provider;

        let provider_cfg = cfg
            .ai
            .providers
            .get(&provider)
            .expect("somehow did not find provider config");

        bar.set_message(format!(
            "Awaiting response from {}",
            provider_cfg.model
        ));

        let mut prompt = build_prompt(&cfg);
        if cfg.ai.include_file_tree {
            prompt.push_str(&gai.get_repo_tree());
        }

        let diffs = build_diffs_string(gai.get_file_diffs_as_str());

        let response = get_response(
            &diffs,
            &prompt,
            provider,
            provider_cfg.to_owned(),
        )
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

        bar.finish_with_message("Done! Received a response");

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

        println!("Response Commits({}):", result.commits.len());
        for commit in &result.commits {
            println!(
                "Prefix: {}",
                commit.get_commit_prefix(
                    cfg.gai.commit_config.capitalize_prefix,
                    cfg.gai.commit_config.include_scope
                )
            );
            println!("--Header: {}", commit.message.header);
            println!("--Body: {}", commit.message.body);
            if gai.stage_hunks {
                println!("--Hunks: {:#?}", commit.hunk_ids);
            } else {
                println!("--Files: {:#?}", commit.files);
            }
            println!();
        }

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

        // todo wrap this in an inner loop
        // or put it in a func so we can retry
        // from THIS prompt
        if selection == 0 {
            println!("Applying Commits...");
            gai.apply_commits(&commits);
        } else if selection == 1 {
            let _ = tui::run_tui(cfg, gai, Some(response)).await;
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
