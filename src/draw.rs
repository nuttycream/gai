use std::{default, error::Error};

use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::utils::GaiLogo;

#[derive(Default)]
pub struct App {
    pub state: State,
}

#[derive(Default)]
pub enum State {
    /// initializing gai:
    /// checks for existing repo
    /// does a diff check
    /// and gathers the data
    /// for the user to send
    #[default]
    Warmup,

    /// state where gai is sending
    /// a request or waiting to
    /// receive the response.
    /// This is usually one continous
    /// moment.
    Pending(PendingType),

    /// state where the user can
    /// either: see what to send
    /// to the AI provider
    /// or what the AI provider has
    /// sent back
    Running,
}

pub enum PendingType {
    Sending,
    Receiving,
}

pub fn run(
    mut terminal: DefaultTerminal,
    app_state: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| render(f, app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Esc => break Ok(()),
                event::KeyCode::Char('q' | 'Q') => break Ok(()),
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame, app_state: &App) {
    match &app_state.state {
        State::Warmup => {
            draw_warmup(frame);
        }
        State::Pending(pt) => {}
        State::Running => {
            draw_running(frame);
        }
    }
}

fn center(
    area: Rect,
    horizontal: Constraint,
    vertical: Constraint,
) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] =
        Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn draw_warmup(frame: &mut Frame) {
    let text = Text::from(vec![Line::from("warming up...")]);

    let [logo_area, text_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    frame.render_widget(GaiLogo::new(), logo_area);
    frame.render_widget(text, text_area);
}

fn draw_running(frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("something")
            .block(Block::new().title("gai").borders(Borders::ALL)),
        layout[0],
    );
    frame.render_widget(
        Paragraph::new("foo").block(
            Block::new().title("status").borders(Borders::ALL),
        ),
        layout[1],
    );
}
