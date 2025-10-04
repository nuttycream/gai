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

use crate::git::{DiffType, HunkDiff};

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

impl SelectedTab {
    pub fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        items: &[String],
        tab_content: &TabContent,
        selected_state: &mut ListState,
    ) {
        self.render_layout(
            area,
            buf,
            items,
            tab_content,
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
        items: &[String],
        content: &TabContent,
        selected_state: &mut ListState,
    ) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ]);
        let [list_area, paragraph_area] = horizontal.areas(area);

        let items: Vec<ListItem> = items
            .iter()
            .map(|item| ListItem::new(item.as_str()))
            .collect();

        let title = match self {
            SelectedTab::Diffs => "Diffs",
            _ => "Commits",
        };

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(title)
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(self.palette().c700),
            )
            .highlight_style(SELECTED_STYLE);

        StatefulWidget::render(list, list_area, buf, selected_state);

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
                                line_diff.content.to_owned(),
                            ),
                        };
                        lines.push(styled_line);
                    }

                    // Optional: add spacing between hunks
                    lines.push(Line::from(""));
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
