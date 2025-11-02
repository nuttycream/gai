use crate::{
    ai::{request::Request, response::Response},
    config::Config,
    git::repo::GaiGit,
    tui::app::{Action, App},
};
use anyhow::Result;
use tokio::sync::mpsc;

pub mod app;
pub mod events;
pub mod keys;
pub mod tabs;
pub mod ui;

use events::{Event, EventHandler};

pub async fn run_tui(
    req: Request,
    cfg: Config,
    gai: GaiGit,
    response: Option<Response>,
) -> Result<()> {
    let mut app = App::new(req, cfg, gai, response);

    let (resp_tx, mut resp_rx) = mpsc::channel(1);

    if app.cfg.tui.auto_request {
        app.send_request(resp_tx.clone()).await;
    }

    let mut terminal = ratatui::init();

    let mut event_handler = EventHandler::new(100);

    while app.running {
        terminal.draw(|f| app.run(f))?;

        tokio::select! {
            Some(event) = event_handler.next() => {
                handle_event(&mut app, event, resp_tx.clone()).await;
            }

            Some(resp) = resp_rx.recv() => {
                app.display_response(resp);
            }
        }
    }

    event_handler.stop().await?;
    ratatui::restore();

    Ok(())
}

async fn handle_event(
    app: &mut App,
    event: Event,
    response_tx: mpsc::Sender<Response>,
) {
    match event {
        Event::Key(key) => {
            if let Some(action) = keys::get_tui_action(key) {
                handle_action(app, action, response_tx).await;
            }
        }
        Event::AppTick => {
            app.on_tick();
        }
        Event::Error => {
            // ignoring for now
            app.running = false;
        }
    }
}

async fn handle_action(
    app: &mut App,
    action: Action,
    response_tx: mpsc::Sender<Response>,
) {
    let ui = &mut app.ui;

    match action {
        Action::Quit => app.running = false,
        Action::ScrollUp => ui.scroll_up(),
        Action::ScrollDown => ui.scroll_down(),
        Action::FocusLeft => ui.focus_left(),
        Action::FocusRight => ui.focus_right(),
        Action::Enter => ui.enter_ui(),
        Action::DiffTab => ui.goto_tab(1),
        Action::OpenAITab => ui.goto_tab(2),
        Action::ClaudeTab => ui.goto_tab(3),
        Action::GeminiTab => ui.goto_tab(4),
        Action::SendRequest => {
            app.send_request(response_tx).await;
        }
        Action::ApplyCommits => {
            app.apply_commits();
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
