pub mod app;
pub mod config;
pub mod consts;
pub mod git;
pub mod keys;
pub mod provider;
pub mod response;
pub mod tabs;
pub mod ui;
pub mod utils;

use anyhow::Result;
use dotenv::dotenv;
use ratatui::crossterm::event::{self, Event};
use tokio::sync::mpsc::{self, Sender};

use crate::{
    app::{Action, App},
    response::Response,
    ui::UI,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let cfg = config::Config::init("config.toml")?;

    let mut gai = git::GaiGit::new(".")?;
    gai.create_diffs(&cfg.files_to_ignore)?;

    let mut app = App::new(cfg, gai);

    let mut terminal = ratatui::init();
    let mut ui = UI::new();

    let (tx, mut rx) = mpsc::channel(3);

    while app.running {
        terminal.draw(|f| ui.render(f, &app))?;

        tokio::select! {
            Ok(event) = async { event::read() } => {
                handle_actions(&mut app, event, &mut ui, tx.clone()).await;
            }

            Some((provider, result)) = rx.recv() => {
                app.responses.insert(provider, result);
            }
        }
    }

    ratatui::restore();

    Ok(())
}

async fn handle_actions(
    app: &mut App,
    event: Event,
    ui: &mut UI,
    tx: Sender<(String, Result<Response, String>)>,
) {
    if let Some(action) = keys::get_tui_action(event) {
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
            _ => {}
        }
    }
}
