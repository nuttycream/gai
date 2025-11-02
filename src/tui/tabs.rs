#![allow(clippy::too_many_arguments)]

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
use throbber_widgets_tui::{Throbber, ThrobberState};

use crate::{
    ai::response::{PrefixType, ResponseCommit},
    git::repo::{DiffType, HunkDiff},
    tui::ui::UIMode,
};

const SELECTED_STYLE: Style = Style::new()
    .bg(tailwind::SLATE.c800)
    .add_modifier(Modifier::BOLD);

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum SelectedTab {
    #[default]
    Diffs,
    Commits,

    Bisect,
    Find,
    Rebase,
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
    Response(ResponseCommit),
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
        is_loading: bool,
        throbber_state: &mut ThrobberState,
        mode: &UIMode,
        content_scroll: u16,
    ) {
        let scroll = if matches!(mode, UIMode::Content) {
            content_scroll
        } else {
            0
        };

        self.render_layout(
            area,
            buf,
            tab_content,
            tab_list,
            selected_state,
            is_loading,
            throbber_state,
            scroll,
            mode,
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
        is_loading: bool,
        throbber_state: &mut ThrobberState,
        scroll: u16,
        mode: &UIMode,
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
                if matches!(self, SelectedTab::Commits) && is_loading
                {
                    self.render_loading(
                        paragraph_area,
                        buf,
                        desc,
                        throbber_state,
                    );
                } else {
                    self.render_description(
                        paragraph_area,
                        buf,
                        desc,
                        scroll,
                        mode,
                    );
                }
            }
            TabContent::Diff(hunk_diffs) => {
                self.render_diff(
                    paragraph_area,
                    buf,
                    hunk_diffs,
                    scroll,
                    mode,
                );
            }
            TabContent::Response(commit) => {
                self.render_response(
                    paragraph_area,
                    buf,
                    commit,
                    scroll,
                    mode,
                );
            }
        }
    }

    fn render_description(
        self,
        area: Rect,
        buf: &mut Buffer,
        desc: &str,
        scroll: u16,
        mode: &UIMode,
    ) {
        let border_style = if matches!(mode, UIMode::Content) {
            self.palette().c400
        } else {
            self.palette().c700
        };

        let paragraph = Paragraph::new(desc.to_owned())
            .block(
                Block::bordered()
                    .title("Content")
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));

        paragraph.render(area, buf);
    }

    fn render_diff(
        self,
        area: Rect,
        buf: &mut Buffer,
        hunk_diffs: &[HunkDiff],
        scroll: u16,
        mode: &UIMode,
    ) {
        let border_style = if matches!(mode, UIMode::Content) {
            self.palette().c400
        } else {
            self.palette().c700
        };

        let mut lines: Vec<Line> = Vec::new();

        for hunk in hunk_diffs {
            lines.push(
                Line::from(hunk.header.clone())
                    .bg(tailwind::BLUE.c900),
            );

            for line_diff in &hunk.line_diffs {
                let styled_line = match line_diff.diff_type {
                    DiffType::Additions => {
                        Line::from(format!("+{}", line_diff.content))
                            .bg(tailwind::GREEN.c950)
                    }
                    DiffType::Deletions => {
                        Line::from(format!("-{}", line_diff.content))
                            .bg(tailwind::RED.c950)
                    }
                    DiffType::Unchanged => {
                        Line::from(format!(" {}", line_diff.content))
                    }
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
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));

        paragraph.render(area, buf);
    }

    fn render_response(
        self,
        area: Rect,
        buf: &mut Buffer,
        commit: &ResponseCommit,
        scroll: u16,
        mode: &UIMode,
    ) {
        let border_style = if matches!(mode, UIMode::Content) {
            self.palette().c400
        } else {
            self.palette().c700
        };

        let mut lines: Vec<Line> = Vec::new();

        let prefix_color = match commit.message.prefix {
            PrefixType::Feat => tailwind::GREEN,
            PrefixType::Fix => tailwind::RED,
            PrefixType::Refactor => tailwind::BLUE,
            PrefixType::Style => tailwind::PURPLE,
            PrefixType::Test => tailwind::YELLOW,
            PrefixType::Docs => tailwind::CYAN,
            PrefixType::Build => tailwind::ORANGE,
            PrefixType::CI => tailwind::INDIGO,
            PrefixType::Ops => tailwind::PINK,
            PrefixType::Chore => tailwind::SLATE,
            PrefixType::Merge => tailwind::VIOLET,
            PrefixType::Revert => tailwind::ROSE,
        };

        let prefix_str =
            format!("{:?}", commit.message.prefix).to_lowercase();
        let breaking_str =
            if commit.message.breaking { "!" } else { "" };
        let scope_str = if !commit.message.scope.is_empty() {
            format!("({})", commit.message.scope)
        } else {
            String::new()
        };

        lines.push(Line::from(vec![
            prefix_str
                .fg(prefix_color.c200)
                .bg(prefix_color.c900)
                .bold(),
            scope_str.fg(tailwind::SLATE.c400).italic(),
            breaking_str.fg(tailwind::RED.c500).bold(),
        ]));
        lines.push(Line::from(""));

        lines.push(
            Line::from("Header").fg(tailwind::SLATE.c500).bold(),
        );
        lines.push(
            Line::from(commit.message.header.clone())
                .fg(tailwind::SLATE.c100),
        );
        lines.push(Line::from(""));

        if !commit.message.body.is_empty() {
            lines.push(
                Line::from("Body").fg(tailwind::SLATE.c500).bold(),
            );
            for body_line in commit.message.body.lines() {
                lines.push(
                    Line::from(body_line).fg(tailwind::SLATE.c300),
                );
            }
            lines.push(Line::from(""));
        }

        if !commit.files.is_empty() {
            lines.push(
                Line::from("Files").fg(tailwind::SLATE.c500).bold(),
            );
            for file in &commit.files {
                lines.push(
                    Line::from(format!("  • {}", file))
                        .fg(tailwind::CYAN.c400),
                );
            }
            lines.push(Line::from(""));
        }

        if !commit.hunk_ids.is_empty() {
            lines.push(
                Line::from("Hunks").fg(tailwind::SLATE.c500).bold(),
            );
            for hunk_id in &commit.hunk_ids {
                lines.push(
                    Line::from(format!("  • {}", hunk_id))
                        .fg(tailwind::AMBER.c400),
                );
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::bordered()
                    .title("Commit Info")
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));

        paragraph.render(area, buf);
    }

    fn render_loading(
        self,
        area: Rect,
        buf: &mut Buffer,
        message: &str,
        throbber_state: &mut ThrobberState,
    ) {
        let block = Block::bordered()
            .title("Loading...")
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .border_style(self.palette().c700);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let throbber = Throbber::default()
            .label(message)
            .style(Style::default().fg(tailwind::CYAN.c400))
            .throbber_style(
                Style::default()
                    .fg(tailwind::CYAN.c500)
                    .add_modifier(Modifier::BOLD),
            )
            .throbber_set(throbber_widgets_tui::BRAILLE_SIX)
            .use_type(throbber_widgets_tui::WhichUse::Spin);

        StatefulWidget::render(
            throbber,
            inner_area,
            buf,
            throbber_state,
        );
    }

    pub const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Diffs => tailwind::GREEN,
            Self::Commits => tailwind::BLUE,
            Self::Bisect => tailwind::EMERALD,
            Self::Find => tailwind::AMBER,
            Self::Rebase => tailwind::INDIGO,
        }
    }
}
