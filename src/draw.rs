use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, palette::tailwind::SLATE},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{response::Response, ui::UI};

const SELECTED_STYLE: Style =
    Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl UI {
    pub fn draw_splash(&self, frame: &mut Frame) {}

    pub fn draw_pending(&self, frame: &mut Frame) {
        let area = center(
            frame.area(),
            Constraint::Length(25),
            Constraint::Length(3),
        );

        let popup = Paragraph::new("sending request...")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::Green))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            );

        frame.render_widget(popup, area);
    }

    pub fn draw_diffview(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .margin(10)
            .split(frame.area());

        let items: Vec<ListItem> = self
            .selection_list
            .iter()
            .map(|s| ListItem::new(s.as_str()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Files")
                    .borders(Borders::ALL)
                    .border_style(if !self.in_content_mode {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::Gray)
                    }),
            )
            .highlight_style(SELECTED_STYLE);

        frame.render_stateful_widget(
            list,
            layout[0],
            &mut self.selected_state,
        );

        let border_style = if self.in_content_mode {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let content = Paragraph::new(self.content_text.to_owned())
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title("Changes")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

        frame.render_widget(content, layout[1]);
    }

    pub fn draw_opsview(
        &self,
        frame: &mut Frame,
        ops: Option<&[Response]>,
    ) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .margin(10)
            .split(frame.area());

        let content =
            Paragraph::new("").wrap(Wrap { trim: false }).block(
                Block::default()
                    .title("Changes")
                    .borders(Borders::ALL),
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
