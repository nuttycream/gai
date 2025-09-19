use std::{
    error::Error,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Modifier, Style, palette::tailwind::SLATE},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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

    current_file: String,
}

#[derive(Default)]
pub enum UIActions {
    #[default]
    None,

    /// remove action
    /// this will ONLY
    /// remove it from gai
    /// and not remove it from git
    /// so if this is triggered on a file
    /// it won't be sent as a diff
    /// to the AI
    Remove,
}

impl UI {
    pub fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        app_state: &mut App,
    ) -> Result<(), Box<dyn Error>> {
        let warmup = Instant::now();

        self.file_paths = app_state.get_file_paths();
        self.file_path_state.select(Some(0));
        self.current_file =
            app_state.get_diff_content(&self.file_paths[0]);

        loop {
            terminal.draw(|f| self.render(f, app_state))?;

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
                            self.update_curr_diff(app_state);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.file_path_state.select_previous();
                            self.update_curr_diff(app_state);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn update_curr_diff(&mut self, app_state: &App) {
        if let Some(selected) = self.file_path_state.selected() {
            if selected < self.file_paths.len() {
                self.current_file = app_state
                    .get_diff_content(&self.file_paths[selected]);
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, app_state: &App) {
        match &app_state.state {
            State::Warmup => {
                draw_warmup(frame);
            }
            State::Pending(pt) => {}
            State::Running => {
                self.draw_running(frame);
            }
        }
    }

    fn draw_running(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .margin(10)
            .split(frame.area());

        let items: Vec<ListItem> = self
            .file_paths
            .iter()
            .map(|path| ListItem::new(path.as_str()))
            .collect();

        let files_list = List::new(items)
            .block(
                Block::default()
                    .title("files")
                    .borders(Borders::RIGHT),
            )
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("-> ");

        frame.render_stateful_widget(
            files_list,
            layout[0],
            &mut self.file_path_state,
        );

        let content = Paragraph::new(self.current_file.as_str())
            .block(
                Block::default()
                    .title("content")
                    .borders(Borders::NONE),
            );

        frame.render_widget(content, layout[1]);
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
