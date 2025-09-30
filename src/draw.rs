use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{
        Color, Modifier, Style, Stylize, palette::tailwind::SLATE,
    },
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Widget,
        Wrap,
    },
};
use strum::{Display, EnumIter, FromRepr};

use crate::{
    app::{App, State},
    response::Response,
    utils::GaiLogo,
};

const SELECTED_STYLE: Style =
    Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(Default)]
pub struct UI {
    file_paths: Vec<String>,
    file_path_state: ListState,
    file_scroll_state: ScrollbarState,

    commit_view_state: ListState,

    current_file: String,
    content_scroll: u16,
    content_scroll_state: ScrollbarState,
    in_content_mode: bool,

    selected_tab: SelectedAI,
}

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedAI {
    #[default]
    Gemini,
    OpenAI,
    Claude,
}

#[derive(Default)]
enum UIActions {
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

impl Widget for SelectedAI {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self {
            SelectedAI::Gemini => todo!(),
            SelectedAI::OpenAI => todo!(),
            SelectedAI::Claude => todo!(),
        }
    }
}

impl SelectedAI {
    pub fn previous(self) -> Self {
        let curr_index = self as usize;
        let prev_index = curr_index.saturating_sub(1);
        Self::from_repr(prev_index).unwrap_or(self)
    }

    pub fn next(self) -> Self {
        let curr_index = self as usize;
        let next_index = curr_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }
}

impl UI {
    pub async fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        app_state: &mut App,
    ) -> Result<()> {
        let warmup = Instant::now();

        self.file_paths = app_state.get_file_paths();
        self.file_path_state.select(Some(0));
        self.current_file =
            app_state.get_diff_content(&self.file_paths[0]);

        self.file_scroll_state =
            ScrollbarState::new(self.file_paths.len());
        self.update_content_scroll();

        loop {
            terminal.draw(|f| self.render(f, app_state))?;

            if matches!(app_state.state, State::Splash)
                && warmup.elapsed() >= Duration::from_secs(2)
                && !app_state.cfg.skip_splash
            {
                app_state.state = State::DiffView { selected: 0 };
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

                        // todo refactor
                        KeyCode::Char('j') | KeyCode::Down => {
                            match &app_state.state {
                                State::DiffView { .. } => {
                                    if self.in_content_mode {
                                        self.content_scroll = self
                                            .content_scroll
                                            .saturating_add(1);
                                        self.update_content_scroll();
                                    } else {
                                        self.file_path_state
                                            .select_next();
                                        self.update_curr_diff(
                                            app_state,
                                        );
                                        self.update_file_scroll();
                                    }
                                }
                                State::OpsView(resp) => {
                                    if self.in_content_mode {
                                        self.content_scroll = self
                                            .content_scroll
                                            .saturating_add(1);
                                        self.update_content_scroll();
                                    } else {
                                        self.commit_view_state
                                            .select_next();
                                        self.update_curr_commit(resp);
                                    }
                                }
                                _ => {}
                            }
                        }

                        KeyCode::Char('k') | KeyCode::Up => {
                            match &app_state.state {
                                State::DiffView { .. } => {
                                    if self.in_content_mode {
                                        self.content_scroll = self
                                            .content_scroll
                                            .saturating_sub(1);
                                        self.update_content_scroll();
                                    } else {
                                        self.file_path_state
                                            .select_previous();
                                        self.update_curr_diff(
                                            app_state,
                                        );
                                        self.update_file_scroll();
                                    }
                                }
                                State::OpsView(resp) => {
                                    if self.in_content_mode {
                                        self.content_scroll = self
                                            .content_scroll
                                            .saturating_sub(1);
                                        self.update_content_scroll();
                                    } else {
                                        self.commit_view_state
                                            .select_previous();
                                        self.update_curr_commit(resp);
                                    }
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Char('p') => {
                            app_state.switch_state(State::Pending);
                            // todo: remove temp force redraw
                            terminal.draw(|f| {
                                self.render(f, app_state)
                            })?;
                            match app_state.send_request().await {
                                Ok(resp) => app_state.switch_state(
                                    State::OpsView(resp),
                                ),
                                Err(e) => panic!(
                                    "failed to send request: {e}"
                                ),
                            }
                        }
                        KeyCode::Char('x') => {
                            match &app_state.state {
                                State::OpsView(response) => {
                                    app_state.apply_ops(response);

                                    break Ok(());
                                }
                                _ => {}
                            }
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

    // todo this needs to moved
    // + commit.message.prefix determine if lowercase atp?
    // maybe make a specific func for it in the commit struct
    fn update_curr_commit(&mut self, resp: &Response) {
        if let Some(selected) = self.commit_view_state.selected() {
            if selected < resp.commits.len() {
                let commit = &resp.commits[selected];
                // use curr file for now
                let prefix = format!("{:?}", commit.message.prefix)
                    .to_lowercase();
                self.current_file = format!(
                    "files to stage:\n{}\ncommit message:\n{}\n",
                    commit
                        .files
                        .iter()
                        .map(|f| format!("  - {}", f))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    format!("{}: {}", prefix, commit.message.message)
                );
                self.content_scroll = 0;
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

    pub fn render(&mut self, frame: &mut Frame, app_state: &App) {
        match &app_state.state {
            State::Splash => {
                draw_splash(frame);
            }
            State::Pending => {
                self.draw_pending(frame);
            }
            State::DiffView { .. } => {
                self.draw_diff_view(frame);
            }
            State::OpsView(resp) => {
                self.draw_ops_view(frame, resp);
            }
        }
    }

    fn draw_pending(&mut self, frame: &mut Frame) {
        let area = center(
            frame.area(),
            Constraint::Length(25),
            Constraint::Length(3),
        );

        let popup = Paragraph::new("sending request...")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::new().yellow())
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(Style::new().red()),
            );
        frame.render_widget(popup, area);
    }

    fn draw_ops_view(&mut self, frame: &mut Frame, resp: &Response) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .margin(5)
            .split(frame.area());

        let commits: Vec<ListItem> = resp
            .commits
            .iter()
            .map(|c| ListItem::new(format!("{:?}", c.message.prefix)))
            .collect();

        let commit_list = List::new(commits)
            .block(
                Block::default()
                    .title("commits")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("->");

        frame.render_stateful_widget(
            commit_list,
            layout[0],
            &mut self.commit_view_state,
        );

        let content = Paragraph::new(self.current_file.as_str())
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title("commit")
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default().fg(Color::DarkGray),
                    ),
            );

        frame.render_widget(content, layout[1]);
    }

    fn draw_diff_view(&mut self, frame: &mut Frame) {
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

        let border_style = if self.in_content_mode {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let files_list = List::new(items)
            .block(
                Block::default()
                    .title("files")
                    .borders(Borders::ALL)
                    .border_style(if !self.in_content_mode {
                        Style::default().fg(Color::Cyan)
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
