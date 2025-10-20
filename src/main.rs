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
use dotenv::dotenv;
use futures::{FutureExt, StreamExt};
use indicatif::ProgressBar;
use std::io::{self, Write};
use std::time::Duration;
use tokio::{
    sync::mpsc::{self, Sender},
    time::interval,
};

use crate::ai::response::ResponseCommit;
use crate::tui::keys;
use crate::{
    ai::{
        provider::{try_claude, try_gemini, try_openai},
        response::Response,
    },
    cli::{Cli, Commands},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    tui::app::{Action, App},
    utils::build_prompt,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let mut cfg = config::Config::init("config.toml")?;

    let args = Cli::parse();

    args.parse_args(&mut cfg);

    let mut gai = GaiGit::new(
        cfg.stage_hunks,
        cfg.ai.capitalize_prefix,
        cfg.ai.include_scope,
    )?;

    gai.create_diffs(&cfg.files_to_truncate)?;
    match args.command {
        Commands::Tui { .. } => run_tui(cfg, gai).await?,
        Commands::Commit { .. } => run_gemini(cfg, gai).await?,
        Commands::Find { .. } => println!("Not yet implemented"),
        Commands::Rebase {} => println!("Not yet implemented"),
    }

    Ok(())
}

async fn run_provider(
    cfg: Config,
    gai: GaiGit,
    resp: &mut Response,
) -> Result<()> {
    println!("Response Commits({}):", resp.commits.len());
    for commit in &resp.commits {
        println!(
            "prefix: {}",
            commit.get_commit_prefix(
                cfg.ai.capitalize_prefix,
                cfg.ai.include_scope
            )
        );
        println!("--header: {}", commit.message.header);
        println!("--body: {}", commit.message.body);
        if gai.stage_hunks {
            println!("--hunks: {:#?}", commit.hunk_ids);
        } else {
            println!("--files: {:#?}", commit.files);
        }
    }
    println!(
        "\n[y/Y] Apply Commit/s\n[e/E] Edit Commit\n[q/Q] Cancel/Quit"
    );

    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    input = input.trim().to_string();

    if input.eq_ignore_ascii_case("n") {
        println!("Quitting");
        return Ok(());
    } else if input.eq_ignore_ascii_case("y") {
        let commits: Vec<GaiCommit> = resp
            .commits
            .iter()
            .map(|resp_commit| {
                GaiCommit::from_response(
                    resp_commit,
                    gai.capitalize_prefix,
                    gai.include_scope,
                )
            })
            .collect();

        gai.apply_commits(&commits);
    } else if input.eq_ignore_ascii_case("e") {
        println!(
            "Select a commit to edit [1 - {}]:",
            resp.commits.len()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        input = input.trim().to_string();
        match input.parse::<i32>() {
            Ok(i) => {
                if (i as usize - 1) < resp.commits.len() {
                    let commit = &mut resp.commits[i as usize - 1];
                    edit_commit(commit)?;
                }
            }
            Err(e) => println!("error with input({input}): {e}"),
        }
    }

    Ok(())
}

fn edit_commit(commit: &mut ResponseCommit) -> Result<()> {
    println!("Selected: {}", commit.message.header);
    println!("Edit:\n[h/H] Header\n[b/B] Body\n[q/Q] Quit");

    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    input = input.trim().to_string();

    // todo i might have to use an external crate
    // like rustyline to edit the string
    // or use inline ratatui
    if input.eq_ignore_ascii_case("h") {
        println!("Editing commit message header...");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        commit.message.header = input.trim().to_string();

        println!("{}", commit.message.header);
    } else if input.eq_ignore_ascii_case("b") {
        println!("Editing commit message body...");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        commit.message.body = input.trim().to_string();

        println!("{}", commit.message.body);
    } else if input.eq_ignore_ascii_case("q") {
        return Ok(());
    }

    Ok(())
}

async fn run_gemini(cfg: Config, gai: GaiGit) -> Result<()> {
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));

    println!("Sending diffs to gemini {}", cfg.ai.gemini.model_name);
    let diffs = build_diffs_string(&gai);
    let prompt = build_full_prompt(&cfg, &gai);

    let gemini = &cfg.ai.gemini;
    let mut resp = try_gemini(
        &prompt,
        &gemini.model_name,
        gemini.max_tokens,
        &diffs,
    )
    .await?;

    bar.finish();

    run_provider(cfg, gai, &mut resp).await
}

async fn run_chatgpt(cfg: Config, gai: GaiGit) -> Result<()> {
    println!("Sending diffs to chatgpt {}", cfg.ai.openai.model_name);
    let diffs = build_diffs_string(&gai);
    let prompt = build_full_prompt(&cfg, &gai);

    let chatgpt = &cfg.ai.openai;
    let mut resp =
        try_openai(&prompt, &chatgpt.model_name, &diffs).await?;

    run_provider(cfg, gai, &mut resp).await
}

async fn run_claude(cfg: Config, gai: GaiGit) -> Result<()> {
    println!("Sending diffs to claude {}", cfg.ai.claude.model_name);
    let diffs = build_diffs_string(&gai);
    let prompt = build_full_prompt(&cfg, &gai);

    let claude = &cfg.ai.claude;
    let mut resp =
        try_claude(&prompt, &claude.model_name, &diffs).await?;

    run_provider(cfg, gai, &mut resp).await
}

async fn run_tui(cfg: Config, gai: GaiGit) -> Result<()> {
    let mut app = App::new(cfg, gai);

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

            Some((provider, result)) = rx.recv() => {
                app.pending.remove(&provider);
                app.responses.insert(provider, result);
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
    tx: Sender<(String, Result<Response, String>)>,
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

fn build_diffs_string(gai: &GaiGit) -> String {
    let mut diffs = String::new();

    for (file, diff) in gai.get_file_diffs_as_str() {
        let file_diff =
            format!("FileName:{}\nContent:{}\n\n", file, diff);
        diffs.push_str(&file_diff);
    }

    diffs
}

fn build_full_prompt(cfg: &Config, gai: &GaiGit) -> String {
    let rules = cfg.ai.build_rules();

    let mut prompt = build_prompt(
        cfg.ai.include_convention,
        &cfg.ai.system_prompt,
        &rules,
        cfg.stage_hunks,
    );

    if cfg.include_file_tree {
        prompt.push_str(&gai.get_repo_tree());
    }

    prompt
}
