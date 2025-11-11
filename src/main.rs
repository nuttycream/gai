pub mod ai;
pub mod auth;
pub mod cli;
pub mod config;
pub mod consts;
pub mod git;
pub mod graph;
pub mod tui;

use anyhow::Result;
use chrono::DateTime;
use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
};
use dialoguer::{Confirm, Password, Select, theme::ColorfulTheme};
use dotenv::dotenv;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{
    io::{Stdout, stdout},
    time::Duration,
};

use crate::{
    ai::{
        request::Request,
        response::{ResponseCommit, get_response},
    },
    auth::{get_token, store_token},
    cli::{Auth, Cli, Commands},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    graph::Arena,
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

            match args.command {
                Commands::Tui { auto_request } => {
                    let bar = create_spinner_bar();
                    let req = build_request(&cfg, &gai, &bar);
                    if auto_request {
                        cfg.tui.auto_request = true;
                    }
                    run_tui(req, cfg, gai, None).await?
                }
                Commands::Commit { skip_confirmation } => {
                    let bar = create_spinner_bar();
                    let req = build_request(&cfg, &gai, &bar);
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
                        let bar = create_spinner_bar();
                        let req = build_request(&cfg, &gai, &bar);
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

fn auth_login() -> Result<()> {
    println!("Opening Browser for https://cli.gai.fyi/login");
    open::that("https://cli.gai.fyi/login")?;
    let token = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Paste Token: ")
        .interact()?;

    println!("Storing token of length: {}", token.len());

    store_token(&token)?;
    Ok(())
}

async fn auth_status(bar: ProgressBar) -> Result<()> {
    bar.set_message("Grabbing Status");
    let token = get_token()?;

    let client = reqwest::Client::new();
    let resp = client
        .get("https://cli.gai.fyi/status")
        .bearer_auth(token)
        .send()
        .await?;

    #[derive(Deserialize, Serialize, Debug)]
    struct Status {
        requests_made: i32,
        expiration: u64,
    }

    let status = resp.json::<Status>().await?;

    bar.finish();

    if let Some(date) =
        DateTime::from_timestamp(status.expiration.try_into()?, 0)
    {
        println!("Requests made: {}/10", status.requests_made);
        println!("Resets at {}", date);
    } else {
        println!("Failed to convert expiration to datetime");
    }

    Ok(())
}

fn clear_auth() -> Result<()> {
    auth::delete_token()?;
    println!("No longer aunthenticated");
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

fn pretty_print_status(
    stdout: &mut Stdout,
    gai: &GaiGit,
) -> Result<()> {
    let mut arena = Arena::new();

    let branch = &gai.get_branch();
    let status = &gai.status;

    let staged_count = gai.staged_len();
    let unstaged_count = gai.unstaged_len();

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("On Branch: {}\n", branch).bold()),
        ResetColor
    )?;

    if unstaged_count == 0 && staged_count == 0 {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("No Diffs".bold()),
            ResetColor
        )?;

        return Ok(());
    }

    if staged_count > 0 {
        let staged_root = arena.new_node("✓ Staged", Color::Green);
        arena.set_count(staged_root, staged_count);

        if !status.s_new.is_empty() {
            let new_node = arena.new_node("New", Color::Green);
            arena.set_count(new_node, status.s_new.len());
            arena.add_child(staged_root, new_node);

            for file in &status.s_new {
                let file_node = arena.new_node(file, Color::Green);
                arena.set_prefix(file_node, "A");
                arena.add_child(new_node, file_node);
            }
        }

        // mod
        if !status.s_modified.is_empty() {
            let modified_node =
                arena.new_node("Modified", Color::Blue);
            arena.set_count(modified_node, status.s_modified.len());
            arena.add_child(staged_root, modified_node);

            for file in &status.s_modified {
                let file_node = arena.new_node(file, Color::Blue);
                arena.set_prefix(file_node, "M");
                arena.add_child(modified_node, file_node);
            }
        }

        // del
        if !status.s_deleted.is_empty() {
            let deleted_node = arena.new_node("Deleted", Color::Red);
            arena.set_count(deleted_node, status.s_deleted.len());
            arena.add_child(staged_root, deleted_node);

            for file in &status.s_deleted {
                let file_node = arena.new_node(file, Color::Red);
                arena.set_prefix(file_node, "D");
                arena.add_child(deleted_node, file_node);
            }
        }

        // ren
        if !status.s_renamed.is_empty() {
            let renamed_node =
                arena.new_node("Renamed", Color::Magenta);
            arena.set_count(renamed_node, status.s_renamed.len());
            arena.add_child(staged_root, renamed_node);

            for (old, new) in &status.s_renamed {
                let label = format!("{} → {}", old, new);
                let file_node = arena.new_node(label, Color::White);
                arena.set_prefix(file_node, "R");
                arena.add_child(renamed_node, file_node);
            }
        }
    }

    if unstaged_count > 0 {
        let unstaged_root =
            arena.new_node("⚠ Unstaged", Color::Yellow);
        arena.set_count(unstaged_root, unstaged_count);

        if !status.u_new.is_empty() {
            let new_node = arena.new_node("New", Color::Green);
            arena.set_count(new_node, status.u_new.len());
            arena.add_child(unstaged_root, new_node);

            for file in &status.u_new {
                let file_node = arena.new_node(file, Color::Green);
                arena.set_prefix(file_node, "?");
                arena.add_child(new_node, file_node);
            }
        }

        if !status.u_modified.is_empty() {
            let modified_node =
                arena.new_node("Modified", Color::Blue);
            arena.set_count(modified_node, status.u_modified.len());
            arena.add_child(unstaged_root, modified_node);

            for file in &status.u_modified {
                let file_node = arena.new_node(file, Color::Blue);
                arena.set_prefix(file_node, "M");
                arena.add_child(modified_node, file_node);
            }
        }

        if !status.u_deleted.is_empty() {
            let deleted_node = arena.new_node("Deleted", Color::Red);
            arena.set_count(deleted_node, status.u_deleted.len());
            arena.add_child(unstaged_root, deleted_node);

            for file in &status.u_deleted {
                let file_node = arena.new_node(file, Color::Red);
                arena.set_prefix(file_node, "D");
                arena.add_child(deleted_node, file_node);
            }
        }

        if !status.u_renamed.is_empty() {
            let renamed_node =
                arena.new_node("Renamed", Color::Magenta);
            arena.set_count(renamed_node, status.u_renamed.len());
            arena.add_child(unstaged_root, renamed_node);

            for (old, new) in &status.u_renamed {
                let label = format!("{} → {}", old, new);
                let file_node = arena.new_node(label, Color::White);
                arena.set_prefix(file_node, "R");
                arena.add_child(renamed_node, file_node);
            }
        }
    }

    arena.print_tree(stdout)?;

    Ok(())
}

fn pretty_print_commits(
    stdout: &mut Stdout,
    commits: &[ResponseCommit],
    cfg: &Config,
    gai: &GaiGit,
) -> Result<()> {
    let mut arena = Arena::new();

    for (i, commit) in commits.iter().enumerate() {
        let prefix = commit.get_commit_prefix(
            cfg.gai.commit_config.capitalize_prefix,
            cfg.gai.commit_config.include_scope,
        );

        let commit_root = arena
            .new_node(format!("Commit {}", i + 1), Color::DarkGrey);

        let prefix_node = arena.new_node(prefix, Color::Green);
        arena.add_child(commit_root, prefix_node);

        let header_node = arena.new_node(
            format!("Header: {}", commit.message.header),
            Color::White,
        );
        arena.add_child(commit_root, header_node);

        if !commit.message.body.is_empty() {
            let body_text = arena.truncate(&commit.message.body, 45);
            let body_node = arena.new_node(
                format!("Body: {}", body_text),
                Color::Blue,
            );
            arena.add_child(commit_root, body_node);
        }

        if gai.stage_hunks {
            let hunks_node = arena.new_node(
                format!("Hunks: {:?}", commit.hunk_ids),
                Color::Magenta,
            );
            arena.add_child(commit_root, hunks_node);
        } else {
            let files_parent =
                arena.new_node("Files", Color::Magenta);
            arena.set_count(files_parent, commit.files.len());
            arena.add_child(commit_root, files_parent);

            for file in &commit.files {
                let file_node = arena.new_node(file, Color::White);
                arena.add_child(files_parent, file_node);
            }
        }
    }

    arena.print_tree(stdout)?;

    Ok(())
}
