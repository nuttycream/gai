use std::{
    error::Error,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style, palette::tailwind::SLATE},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
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
    file_scroll_state: ScrollbarState,

    current_file: String,
    content_scroll: u16,
    content_scroll_state: ScrollbarState,
    in_content_mode: bool,
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

        self.file_scroll_state =
            ScrollbarState::new(self.file_paths.len());
        self.update_content_scroll();

        if app_state.cfg.skip_splash {
            app_state.state = State::Running;
        }

        loop {
            terminal.draw(|f| self.render(f, app_state))?;

            if matches!(app_state.state, State::Warmup)
                && warmup.elapsed() >= Duration::from_secs(2)
                && !app_state.cfg.skip_splash
            {
                app_state.state = State::Running;
            }

            if poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => break Ok(()),
                        KeyCode::Char('q' | 'Q') => break Ok(()),
                        KeyCode::Char('h') | KeyCode::Left => {
                            if self.in_content_mode {
                                self.in_content_mode = false;
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if !self.in_content_mode {
                                self.in_content_mode = true;
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if self.in_content_mode {
                                self.content_scroll = self
                                    .content_scroll
                                    .saturating_add(1);
                                self.update_content_scroll();
                            } else {
                                self.file_path_state.select_next();
                                self.update_curr_diff(app_state);
                                self.update_file_scroll();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if self.in_content_mode {
                                self.content_scroll = self
                                    .content_scroll
                                    .saturating_sub(1);
                                self.update_content_scroll();
                            } else {
                                self.file_path_state
                                    .select_previous();
                                self.update_curr_diff(app_state);
                                self.update_file_scroll();
                            }
                        }
                        KeyCode::Char('p') => {
                            app_state.send_request();
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
                self.content_scroll = 0;
                self.in_content_mode = false;
                self.update_content_scroll();
            }
        }
    }

    fn update_file_scroll(&mut self) {
        if let Some(selected) = self.file_path_state.selected() {
            self.file_scroll_state =
                self.file_scroll_state.position(selected);
        }
    }

    fn update_content_scroll(&mut self) {
        let height = self.current_file.lines().count().max(1);
        self.content_scroll_state = ScrollbarState::new(height)
            .position(self.content_scroll as usize);
    }

    fn render(&mut self, frame: &mut Frame, app_state: &App) {
        match &app_state.state {
            State::Warmup => {
                draw_splash(frame);
            }
            State::Pending => {}
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
            .margin(1)
            .split(frame.area());

        let items: Vec<ListItem> = self
            .file_paths
            .iter()
            .map(|path| ListItem::new(path.as_str()))
            .collect();

        let border_style = if self.in_content_mode {
            Style::default().fg(Color::LightGreen)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let files_list = List::new(items)
            .block(
                Block::default()
                    .title("files")
                    .borders(Borders::ALL)
                    .border_style(if !self.in_content_mode {
                        Style::default().fg(Color::LightGreen)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }),
            )
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("-> ");

        frame.render_stateful_widget(
            files_list,
            layout[0],
            &mut self.file_path_state,
        );

        let content_lines: Vec<&str> =
            self.current_file.lines().collect();
        let visible_content = if content_lines.len()
            > self.content_scroll as usize
        {
            content_lines[self.content_scroll as usize..].join("\n")
        } else {
            String::new()
        };

        let content = Paragraph::new(visible_content).block(
            Block::default()
                .title("changes")
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(content, layout[1]);

        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            layout[1],
            &mut self.content_scroll_state,
        );
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

fn draw_splash(frame: &mut Frame) {
    let area = center(
        frame.area(),
        Constraint::Length(32),
        Constraint::Length(32),
    );

    frame.render_widget(GaiLogo::new(), area);
}
