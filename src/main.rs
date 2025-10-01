pub mod app;
pub mod config;
pub mod consts;
pub mod draw;
pub mod git;
pub mod keys;
pub mod provider;
pub mod response;
pub mod ui;
pub mod utils;

use anyhow::Result;
use dotenv::dotenv;
use ratatui::crossterm::event::{self, Event};

use crate::{
    app::{Action, App},
    ui::UI,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cfg = config::Config::init("config.toml")?;

    let mut gai = git::GaiGit::new(".")?;
    gai.create_diffs(&cfg.files_to_ignore)?;

    let state = if cfg.skip_splash {
        app::State::DiffView { selected: 0 }
    } else {
        app::State::Splash
    };

    let mut app = App {
        running: true,
        state,
        cfg,
        gai,
        ops: None,
    };

    let mut terminal = ratatui::init();
    let mut ui = UI::default();

    while app.running {
        terminal.draw(|f| ui.render(f, &app))?;

        tokio::select! {
            Ok(event) = async { event::read() } => {
                handle_actions(&mut app, event, &mut ui);
            }
        }
    }

    ratatui::restore();

    Ok(())
}

fn handle_actions(app: &mut App, event: Event, ui: &mut UI) {
    if let Some(action) = keys::get_tui_action(event, &app.state) {
        match action {
            Action::Quit => app.running = false,
            Action::ScrollUp => ui.scroll_up(&app),
            Action::ScrollDown => ui.scroll_down(&app),
            Action::FocusLeft => ui.focus_left(&app),
            Action::FocusRight => ui.focus_right(&app),
            _ => {}
        }
    }
}
