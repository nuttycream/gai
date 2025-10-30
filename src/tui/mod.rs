use std::time::Duration;

use crate::{
    ai::response::Response,
    config::Config,
    git::repo::GaiGit,
    tui::app::{Action, App},
};
use anyhow::Result;
use crossterm::event::{
    Event as CrossTermEvent, EventStream, KeyEventKind,
};
use futures::{FutureExt, StreamExt};
use tokio::{sync::mpsc, time::interval};

pub mod app;
pub mod keys;
pub mod tabs;
pub mod ui;

// todo sending a request hangs as soon as you press it.
// not sure why, might be because of the extra other funcs
// and not specificically get_response() will look into
// this in the future
// for now let's focus on more pressing issues lol
pub async fn run_tui(
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
