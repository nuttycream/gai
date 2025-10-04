use crate::app::Action;

use crossterm::event::{Event, KeyCode};

pub fn get_tui_action(event: Event) -> Option<Action> {
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

            KeyCode::Char('d') => Some(Action::RemoveCurrentSelected),

            // todo; needs to be selected tab aware
            // here or in main.rs
            KeyCode::Char('p') => Some(Action::SendRequest),
            KeyCode::Char('x') => Some(Action::ApplyCommits),

            KeyCode::Char('1') => Some(Action::DiffTab),
            KeyCode::Char('2') => Some(Action::OpenAITab),
            KeyCode::Char('3') => Some(Action::ClaudeTab),
            KeyCode::Char('4') => Some(Action::GeminiTab),

            _ => None,
        },

        _ => None,
    }
}
