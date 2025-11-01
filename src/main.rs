pub mod ai;
pub mod cli;
pub mod config;
pub mod consts;
pub mod git;
pub mod tui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use dotenv::dotenv;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    io::{Stdout, stdout},
    time::Duration,
};

use crate::{
    ai::{
        request::Request,
        response::{ResponseCommit, get_response},
    },
    cli::{Cli, Commands},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    tui::run_tui,
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

    let mut req = Request::default();

    req.build_prompt(&cfg, &gai);
    req.build_diffs_string(gai.get_file_diffs_as_str());

    match args.command {
        Commands::Tui { .. } => run_tui(req, cfg, gai, None).await?,
        Commands::Commit { skip_confirmation } => {
            run_commit(req, cfg, gai, skip_confirmation).await?
        }
        Commands::Find { .. } => println!("Not yet implemented"),
        Commands::Rebase {} => println!("Not yet implemented"),
        Commands::Bisect {} => println!("Not yet implemented"),
    }

    Ok(())
}

async fn run_commit(
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

        let mut stdout = stdout();

        pretty_print_status(&mut stdout, &gai);

        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(Duration::from_millis(80));
        bar.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&[
                    "⣼", "⣹", "⢻", "⠿", "⡟", "⣏", "⣧", "⣶",
                ]),
        );

        bar.set_message(format!(
            "Awaiting response from {}",
            provider_cfg.model
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
            "Done! Received {} Commit{}\n",
            result.commits.len(),
            if result.commits.len() == 1 { "" } else { "s" }
        );

        bar.finish_with_message(finished_msg);

        pretty_print_commits(
            &mut stdout,
            &result.commits,
            &cfg,
            &gai,
        );

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

fn pretty_print_status(stdout: &mut Stdout, gai: &GaiGit) {
    let status = gai.get_repo_status();
    if status.is_empty() {
        return;
    }

    let lines: Vec<&str> = status.lines().collect();
    let staged: Vec<&str> = lines
        .iter()
        .filter(|l| {
            !l.starts_with(' ')
                && !l.starts_with("Staged")
                && !l.starts_with("Unstaged")
                && !l.is_empty()
        })
        .copied()
        .collect();
    let unstaged: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with(" "))
        .copied()
        .collect();

    execute!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print("Status: "),
        ResetColor
    )
    .unwrap();

    if !staged.is_empty() {
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print(format!("{} staged", staged.len())),
            ResetColor
        )
        .unwrap();
    }

    if !unstaged.is_empty() {
        if !staged.is_empty() {
            execute!(stdout, Print(", "), ResetColor).unwrap();
        }
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(format!("{} unstaged", unstaged.len())),
            ResetColor
        )
        .unwrap();
    }

    println!();
}

fn pretty_print_commits(
    stdout: &mut Stdout,
    commits: &[ResponseCommit],
    cfg: &Config,
    gai: &GaiGit,
) {
    println!();

    for (i, commit) in commits.iter().enumerate() {
        let prefix = commit.get_commit_prefix(
            cfg.gai.commit_config.capitalize_prefix,
            cfg.gai.commit_config.include_scope,
        );

        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("Commit {} --------\n", i + 1)),
            ResetColor
        )
        .unwrap();

        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("→ "),
            SetForegroundColor(Color::White),
            Print(format!("{}\n", prefix.bold())),
            ResetColor
        )
        .unwrap();

        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("  Header: "),
            ResetColor,
            Print(format!("{}\n", commit.message.header)),
        )
        .unwrap();

        if !commit.message.body.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print("  Body:\n"),
                ResetColor,
                Print(format!("{}\n", commit.message.body)),
            )
            .unwrap();
        }

        if gai.stage_hunks {
            execute!(
                stdout,
                SetForegroundColor(Color::Magenta),
                Print("  Hunks:  "),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("{:?}\n", commit.hunk_ids)),
                ResetColor
            )
            .unwrap();
        } else {
            execute!(
                stdout,
                SetForegroundColor(Color::Magenta),
                Print("  Files:  "),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("{:?}\n", commit.files)),
                ResetColor
            )
            .unwrap();
        }

        println!();
    }
}
