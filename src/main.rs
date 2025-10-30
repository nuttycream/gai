pub mod ai;
pub mod cli;
pub mod config;
pub mod consts;
pub mod git;
pub mod tui;
pub mod utils;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{
    Event as CrossTermEvent, EventStream, KeyEventKind,
};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use dotenv::dotenv;
use futures::{FutureExt, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::{
    sync::mpsc::{self},
    time::interval,
};

use crate::{
    ai::response::{Response, get_response},
    cli::{Cli, Commands},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    tui::{
        app::{Action, App},
        keys,
    },
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
        Commands::Tui { .. } => run_tui(cfg, gai, None).await?,
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
            let _ = run_tui(cfg, gai, Some(response)).await;
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

// todo sending a request hangs as soon as you press it.
// not sure why, might be because of the extra other funcs
// and not specificically get_response() will look into
// this in the future
// for now let's focus on more pressing issues lol
async fn run_tui(
    cfg: Config,
    gai: GaiGit,
    response: Option<Response>,
) -> Result<()> {
    let mut app = App::new(cfg, gai, response);

    let mut terminal = ratatui::init();

    let (tx, mut rx) = mpsc::channel(3);

    let mut reader = EventStream::new();
    let mut interval = interval(Duration::from_millis(100));

    while app.running {
        let delay = interval.tick();
        terminal.draw(|f| app.run(f))?;

        tokio::select! {
            maybe_event = reader.next().fuse() => {
                if let Some(Ok(event)) = maybe_event {
                    handle_actions(&mut app, event, tx.clone()).await;
                }
            }

            Some(resp) = rx.recv() => {
                app.response = Some(resp);
            }

            _ = delay => {
            }
        }
    }

    ratatui::restore();

    Ok(())
}

async fn handle_actions(
    app: &mut App,
    event: CrossTermEvent,
    tx: mpsc::Sender<Response>,
) {
    // this is somewhat jank, but from the ratatui docs
    // it seems to be the better solution, though
    // we dont necessarily have an EventHandler that
    // may come in the future. that way we can pass this along
    if let CrossTermEvent::Key(key) = event
        && key.kind == KeyEventKind::Press
    {
        let pressed = CrossTermEvent::Key(key);
        if let Some(action) = keys::get_tui_action(pressed) {
            let ui = &mut app.ui;
            match action {
                Action::Quit => app.running = false,
                Action::ScrollUp => ui.scroll_up(),
                Action::ScrollDown => ui.scroll_down(),
                Action::FocusLeft => ui.focus_left(),
                Action::FocusRight => ui.focus_right(),
                Action::DiffTab => ui.goto_tab(1),
                Action::OpenAITab => ui.goto_tab(2),
                Action::ClaudeTab => ui.goto_tab(3),
                Action::GeminiTab => ui.goto_tab(4),
                Action::SendRequest => {
                    app.send_request(tx).await;
                }
                Action::ApplyCommits => {
                    app.apply_commits();
                    // for now just exit right after
                    // applying commits
                    app.running = false;
                }
                Action::RemoveCurrentSelected => {
                    app.remove_selected();
                }
                Action::TruncateCurrentSelected => {
                    app.truncate_selected();
                }
                _ => {}
            }
        }
    }
}
