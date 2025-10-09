use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize, palette::tailwind},
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, ListState, Padding,
        Paragraph, StatefulWidget, Widget, Wrap,
    },
};
use strum::{Display, EnumIter, FromRepr};

use crate::git::repo::{DiffType, HunkDiff};

const SELECTED_STYLE: Style = Style::new()
    .bg(tailwind::SLATE.c800)
    .add_modifier(Modifier::BOLD);

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum SelectedTab {
    #[default]
    Diffs,
    OpenAI,
    Claude,
    Gemini,
}

/// wrapper to determine
/// if we should display
/// plain strings (such as a
/// commit message desc) or structured
/// diffs, imo, i think this is fine
/// compared to what i was doing before
pub enum TabContent {
    Description(String),
    Diff(Vec<HunkDiff>),
}

/// when we want to display
/// failed hunks/files
/// OR
/// truncated files
pub struct TabList {
    pub main: Vec<String>,
    pub secondary: Option<Vec<String>>,

    pub main_title: String,
    pub secondary_title: Option<String>,
}

impl SelectedTab {
    pub fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        tab_content: &TabContent,
        tab_list: &TabList,
        selected_state: &mut ListState,
    ) {
        self.render_layout(
            area,
            buf,
            tab_content,
            tab_list,
            selected_state,
        );
    }

    /// Get the previous tab, if there is no previous tab return the current tab.
    pub fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    pub fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    pub fn find_tab(self, tab: usize) -> Self {
        Self::from_repr(tab).unwrap_or(self)
    }

    pub fn title(self) -> Line<'static> {
        let idx = self as usize + 1;

        format!(" [{idx}] {self} ")
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c950)
            .into()
    }

    pub fn render_layout(
        self,
        area: Rect,
        buf: &mut Buffer,
        content: &TabContent,
        tab_list: &TabList,
        selected_state: &mut ListState,
    ) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ]);
        let [list_area, paragraph_area] = horizontal.areas(area);

        let items: Vec<ListItem> = tab_list
            .main
            .iter()
            .map(|item| ListItem::new(item.as_str()))
            .collect();

        if let Some(secondary) = &tab_list.secondary {
            let with_secondary = Layout::vertical([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]);

            let [primary_area, secondary_area] =
                with_secondary.areas(list_area);

            let primary_list = List::new(items)
                .block(
                    Block::bordered()
                        .title(tab_list.main_title.to_owned())
                        .borders(Borders::ALL)
                        .padding(Padding::horizontal(1))
                        .border_style(self.palette().c700),
                )
                .highlight_style(SELECTED_STYLE);

            StatefulWidget::render(
                primary_list,
                primary_area,
                buf,
                selected_state,
            );

            let secondary_items: Vec<ListItem> = secondary
                .iter()
                .map(|item| ListItem::new(item.as_str()))
                .collect();

            let secondary_list = List::new(secondary_items)
                .block(
                    Block::bordered()
                        .title(
                            tab_list
                                .secondary_title
                                .to_owned()
                                .unwrap(),
                        )
                        .borders(Borders::ALL)
                        .padding(Padding::horizontal(1))
                        .border_style(self.palette().c700),
                )
                .highlight_style(SELECTED_STYLE);

            Widget::render(secondary_list, secondary_area, buf);
        } else {
            let list = List::new(items)
                .block(
                    Block::bordered()
                        .title(tab_list.main_title.to_owned())
                        .borders(Borders::ALL)
                        .padding(Padding::horizontal(1))
                        .border_style(self.palette().c700),
                )
                .highlight_style(SELECTED_STYLE);

            StatefulWidget::render(
                list,
                list_area,
                buf,
                selected_state,
            );
        }

        match content {
            TabContent::Description(desc) => {
                let paragraph = Paragraph::new(desc.to_owned())
                    .block(
                        Block::bordered()
                            .title("Content")
                            .borders(Borders::ALL)
                            .padding(Padding::horizontal(1))
                            .border_style(self.palette().c700),
                    )
                    .wrap(Wrap { trim: false });

                paragraph.render(paragraph_area, buf);
            }
            TabContent::Diff(hunk_diffs) => {
                let mut lines: Vec<Line> = Vec::new();

                for hunk in hunk_diffs {
                    lines.push(
                        Line::from(hunk.header.clone())
                            .bg(tailwind::BLUE.c900),
                    );

                    for line_diff in &hunk.line_diffs {
                        let styled_line = match line_diff.diff_type {
                            DiffType::Additions => Line::from(
                                format!("+{}", line_diff.content),
                            )
                            .bg(tailwind::GREEN.c950),
                            DiffType::Deletions => Line::from(
                                format!("-{}", line_diff.content),
                            )
                            .bg(tailwind::RED.c950),
                            DiffType::Unchanged => Line::from(
                                format!(" {}", line_diff.content),
                            ),
                        };
                        lines.push(styled_line);
                    }
                }

                let paragraph = Paragraph::new(lines)
                    .block(
                        Block::bordered()
                            .title("Content")
                            .borders(Borders::ALL)
                            .padding(Padding::horizontal(1))
                            .border_style(self.palette().c700),
                    )
                    .wrap(Wrap { trim: false });

                paragraph.render(paragraph_area, buf);
            }
        }
    }

    pub const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Diffs => tailwind::GREEN,
            Self::OpenAI => tailwind::GRAY,
            Self::Claude => tailwind::ORANGE,
            Self::Gemini => tailwind::CYAN,
        }
    }
}
