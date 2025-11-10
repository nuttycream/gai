pub mod ai;
pub mod auth;
pub mod cli;
pub mod config;
pub mod consts;
pub mod git;
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
                        println!("{:#?}", req);
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

// llm generated from the boring Status []
// its a lot prettier lol
fn pretty_print_status(
    stdout: &mut Stdout,
    gai: &GaiGit,
) -> Result<()> {
    let branch = &gai.get_branch();
    let status = &gai.status;

    let staged_count = gai.staged_len();
    let unstaged_count = gai.unstaged_len();

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("On Branch: {}\n", branch)),
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
        execute!(
            stdout,
            SetForegroundColor(Color::Green),
            Print("├─ ✓ Staged"),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(" [{}]\n", staged_count)),
            ResetColor
        )?;

        let mut staged_sections = Vec::new();
        if !status.s_new.is_empty() {
            staged_sections.push((
                "New",
                &status.s_new,
                Color::Green,
                "A",
            ));
        }

        if !status.s_modified.is_empty() {
            staged_sections.push((
                "Modified",
                &status.s_modified,
                Color::Blue,
                "M",
            ));
        }

        if !status.s_deleted.is_empty() {
            staged_sections.push((
                "Deleted",
                &status.s_deleted,
                Color::Red,
                "D",
            ));
        }

        let staged_last_idx = staged_sections.len() - 1;

        for (idx, (label, files, color, prefix)) in
            staged_sections.iter().enumerate()
        {
            let is_last =
                idx == staged_last_idx && status.s_renamed.is_empty();

            let branch =
                if is_last { "└──" } else { "├──" };

            let continuation = if is_last { "   " } else { "│  " };

            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(format!("│  {} ", branch)),
                SetForegroundColor(*color),
                Print(format!("{} ", label)),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("[{}]\n", files.len())),
                ResetColor
            )?;

            for (file_idx, file) in files.iter().enumerate() {
                let is_last_file = file_idx == files.len() - 1;
                let file_branch =
                    if is_last_file { "└─" } else { "├─" };

                execute!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!(
                        "│  {}  {} ",
                        continuation, file_branch
                    )),
                    SetForegroundColor(*color),
                    Print(format!("{} ", prefix)),
                    ResetColor,
                    Print(format!("{}\n", file)),
                )?;
            }
        }

        if !status.s_renamed.is_empty() {
            let is_last = unstaged_count == 0;
            let branch =
                if is_last { "└──" } else { "├──" };
            let continuation = if is_last { "   " } else { "│  " };

            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(format!("│  {} ", branch)),
                SetForegroundColor(Color::Magenta),
                Print("Renamed ".to_string()),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("[{}]\n", status.s_renamed.len())),
                ResetColor
            )?;

            for (file_idx, (old, new)) in
                status.s_renamed.iter().enumerate()
            {
                let is_last_file =
                    file_idx == status.s_renamed.len() - 1;
                let file_branch =
                    if is_last_file { "└─" } else { "├─" };

                execute!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!(
                        "│  {}  {} ",
                        continuation, file_branch
                    )),
                    SetForegroundColor(Color::Magenta),
                    Print("R "),
                    SetForegroundColor(Color::DarkGrey),
                    Print(old.to_string()),
                    Print(" → "),
                    ResetColor,
                    Print(format!("{}\n", new)),
                )?;
            }
        }
    }

    if unstaged_count > 0 {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("└─ ⚠ Unstaged"),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(" [{}]\n", unstaged_count)),
            ResetColor
        )?;

        let mut unstaged_sections = Vec::new();
        if !status.u_new.is_empty() {
            unstaged_sections.push((
                "New",
                &status.u_new,
                Color::Green,
                "?",
            ));
        }
        if !status.u_modified.is_empty() {
            unstaged_sections.push((
                "Modified",
                &status.u_modified,
                Color::Blue,
                "M",
            ));
        }
        if !status.u_deleted.is_empty() {
            unstaged_sections.push((
                "Deleted",
                &status.u_deleted,
                Color::Red,
                "D",
            ));
        }

        let unstaged_last_idx = unstaged_sections.len() - 1;

        for (idx, (label, files, color, prefix)) in
            unstaged_sections.iter().enumerate()
        {
            let is_last = idx == unstaged_last_idx
                && status.u_renamed.is_empty();

            let branch =
                if is_last { "└──" } else { "├──" };
            let continuation = if is_last { "   " } else { "│  " };

            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(format!("   {} ", branch)),
                SetForegroundColor(*color),
                Print(format!("{} ", label)),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("[{}]\n", files.len())),
                ResetColor
            )?;

            for (file_idx, file) in files.iter().enumerate() {
                let is_last_file = file_idx == files.len() - 1;
                let file_branch =
                    if is_last_file { "└─" } else { "├─" };

                execute!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!(
                        "   {}  {} ",
                        continuation, file_branch
                    )),
                    SetForegroundColor(*color),
                    Print(format!("{} ", prefix)),
                    ResetColor,
                    Print(format!("{}\n", file)),
                )?;
            }
        }

        if !status.u_renamed.is_empty() {
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print("   └── "),
                SetForegroundColor(Color::Magenta),
                Print("Renamed ".to_string()),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("[{}]\n", status.u_renamed.len())),
                ResetColor
            )?;

            for (file_idx, (old, new)) in
                status.u_renamed.iter().enumerate()
            {
                let is_last_file =
                    file_idx == status.u_renamed.len() - 1;
                let file_branch =
                    if is_last_file { "└─" } else { "├─" };

                execute!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("       {} ", file_branch)),
                    SetForegroundColor(Color::Magenta),
                    Print("R "),
                    SetForegroundColor(Color::DarkGrey),
                    Print(old.to_string()),
                    Print(" → "),
                    ResetColor,
                    Print(format!("{}\n", new)),
                )?;
            }
        }
    }

    Ok(())
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
