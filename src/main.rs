pub mod ai;
pub mod app;
pub mod cli;
pub mod config;
pub mod consts;
pub mod events;
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
use rig::{
    client::{CompletionClient, ProviderClient},
    providers::gemini::{
        self,
        completion::gemini_api_types::{
            AdditionalParameters, GenerationConfig,
        },
    },
};
use std::io::{self, Write};
use std::time::Duration;
use tokio::{
    sync::mpsc::{self, Sender},
    time::interval,
};

use crate::{
    ai::response::Response,
    app::{Action, App},
    cli::{Cli, Commands},
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    utils::build_prompt,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let mut cfg = config::Config::init("config.toml")?;

    let args = Cli::parse();

    args.parse_args(&mut cfg);

    let mut gai = GaiGit::new(
        ".",
        cfg.stage_hunks,
        cfg.ai.capitalize_prefix,
        cfg.ai.include_scope,
    )?;

    gai.create_diffs(&cfg.files_to_truncate)?;
    match args.command {
        Commands::Tui => run_tui(cfg, gai).await?,
        Commands::Gemini => run_cli(cfg, gai).await?,
        _ => {}
    }

    Ok(())
}

async fn run_cli(cfg: Config, gai: GaiGit) -> Result<()> {
    println!("Sending diffs...");

    let mut diffs = String::new();
    for (file, diff) in gai.get_file_diffs_as_str() {
        diffs.push_str(&format!("File:{}\n{}\n", file, diff));
    }

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

    // gemini for now
    let client = gemini::Client::from_env();
    let gen_cfg = GenerationConfig {
        max_output_tokens: Some(cfg.ai.gemini.max_tokens),
        ..Default::default()
    };

    let param_cfg =
        AdditionalParameters::default().with_config(gen_cfg);

    let extractor = client
        .extractor::<Response>(&cfg.ai.gemini.model_name)
        .preamble(&prompt)
        .additional_params(serde_json::to_value(param_cfg)?)
        .build();

    let resp = extractor.extract(diffs).await?;

    println!("response commits:");
    for commit in &resp.commits {
        println!(
            "prefix: {}",
            commit.get_commit_prefix(
                cfg.ai.capitalize_prefix,
                cfg.ai.include_scope
            )
        );
        println!("  desc: {}", commit.message.description);
        println!("  files: {:?}", commit.files);
    }

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

    print!("\napply commits? [y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("no.");
        return Ok(());
    }

    gai.apply_commits(&commits);

    Ok(())
}

async fn run_tui(cfg: Config, gai: GaiGit) -> Result<()> {
    let mut app = App::new(cfg, gai);

    let mut terminal = ratatui::init();

    // gonna leave channels here for now
    // we should only handle one ai response
    // and just make it synchronous for the cli
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
        if let Some(action) = events::keys::get_tui_action(pressed) {
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
