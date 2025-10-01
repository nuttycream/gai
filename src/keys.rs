use crate::app::{Action, State};

use ratatui::crossterm::event::{Event, KeyCode};

pub fn get_tui_action(event: Event, state: &State) -> Option<Action> {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Esc => Some(Action::Quit),

            KeyCode::Char('q' | 'Q') => Some(Action::Quit),

            KeyCode::Char('k') | KeyCode::Up => {
                Some(Action::ScrollUp)
            }

            KeyCode::Char('j') | KeyCode::Down => {
                Some(Action::ScrollDown)
            }

            KeyCode::Char('h') | KeyCode::Left => {
                Some(Action::FocusLeft)
            }

            KeyCode::Char('l') | KeyCode::Right => {
                Some(Action::FocusRight)
            }

            KeyCode::Char('p')
                if matches!(state, State::DiffView { .. }) =>
            {
                Some(Action::SendRequest)
            }

            KeyCode::Char('x')
                if matches!(state, State::OpsView(_)) =>
            {
                Some(Action::ApplyCommits)
            }

            _ => None,
        },

        _ => None,
    }
}
