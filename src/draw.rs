use std::{
    error::Error,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Modifier, Style, palette::tailwind::SLATE},
    widgets::{Block, Borders, ListState, Paragraph},
};

use crate::{
    app::{App, State},
    utils::GaiLogo,
};

const SELECTED_STYLE: Style =
    Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(Default)]
pub struct UI {
    file_paths: Vec<String>,
    file_path_state: ListState,

    file_contents: Vec<String>,
    current_file: String,
}

impl UI {
    pub fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        app_state: &mut App,
    ) -> Result<(), Box<dyn Error>> {
        let warmup = Instant::now();

        loop {
            terminal.draw(|f| render(f, app_state))?;

            if matches!(app_state.state, State::Warmup)
                && warmup.elapsed() >= Duration::from_secs(2)
            {
                app_state.state = State::Running;
            }

            if poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => break Ok(()),
                        KeyCode::Char('q' | 'Q') => break Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => {
                            self.file_path_state.select_next();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.file_path_state.select_previous();
                        }
                        _ => {}
                    }
                }
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
    let area = center(
        frame.area(),
        Constraint::Length(32),
        Constraint::Length(32),
    );

    frame.render_widget(GaiLogo::new(), area);
}

fn draw_running(frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ])
        .margin(10)
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("something")
            .block(Block::new().title("files").borders(Borders::ALL)),
        layout[0],
    );
    frame.render_widget(
        Paragraph::new("foo").block(
            Block::new().title("content").borders(Borders::ALL),
        ),
        layout[1],
    );
}
