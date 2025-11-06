use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Color,
    text::{Line, Text},
    widgets::{ListState, Tabs, Widget},
};
use strum::IntoEnumIterator;
use throbber_widgets_tui::ThrobberState;

use crate::tui::tabs::{SelectedTab, TabContent, TabList};

#[derive(Default)]
pub struct UI {
    pub selected_tab: SelectedTab,
    pub selected_state: ListState,

    pub throbber_state: ThrobberState,
    pub mode: UIMode,
    pub content_scroll: u16,
}

#[derive(Default)]
pub enum UIMode {
    #[default]
    TabNavigation,
    Content,

    // todo, special mode
    // to edit commit message
    // etc, prolly a popup
    Edit,
}

impl UI {
    pub fn new() -> Self {
        let mut selected_state = ListState::default();
        selected_state.select_first();

        Self {
            selected_tab: SelectedTab::Diffs,
            selected_state,
            throbber_state: ThrobberState::default(),
            mode: UIMode::TabNavigation,
            content_scroll: 0,
        }
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        tab_content: &TabContent,
        tab_list: &TabList,
        is_loading: bool,
    ) {
        use Constraint::{Length, Min};
        let vertical =
            Layout::vertical([Length(1), Min(0), Length(2)])
                .margin(5);
        let [header_area, inner_area, footer_area] =
            vertical.areas(frame.area());

        self.render_tabs(header_area, frame.buffer_mut());

        self.selected_tab.render(
            inner_area,
            frame.buffer_mut(),
            tab_content,
            tab_list,
            &mut self.selected_state,
            is_loading,
            &mut self.throbber_state,
            &self.mode,
            self.content_scroll,
        );

        self.render_footer(footer_area, frame.buffer_mut());
    }

    pub fn scroll_up(&mut self) {
        match self.mode {
            UIMode::TabNavigation => {
                self.selected_state.select_previous()
            }
            UIMode::Content => {
                self.content_scroll =
                    self.content_scroll.saturating_sub(1)
            }
            _ => {}
        }
    }

    pub fn scroll_down(&mut self) {
        match self.mode {
            UIMode::TabNavigation => {
                self.selected_state.select_next()
            }
            UIMode::Content => {
                self.content_scroll =
                    self.content_scroll.saturating_add(1)
            }
            _ => {}
        }
    }

    pub fn focus_left(&mut self) {
        self.selected_tab = self.selected_tab.previous();
        self.mode = UIMode::TabNavigation;
        self.content_scroll = 0;
    }

    pub fn focus_right(&mut self) {
        self.selected_tab = self.selected_tab.next();
        self.mode = UIMode::TabNavigation;
        self.content_scroll = 0;
    }

    pub fn goto_tab(&mut self, tab: usize) {
        self.selected_tab =
            self.selected_tab.find_tab(tab.saturating_sub(1));
        self.mode = UIMode::TabNavigation;
        self.content_scroll = 0;
    }

    pub fn enter_ui(&mut self) {
        match self.selected_tab {
            SelectedTab::Diffs | SelectedTab::Commits => {
                if self.selected_state.selected().is_none() {
                    return;
                }

                self.mode = if matches!(self.mode, UIMode::Content) {
                    self.content_scroll = 0;
                    UIMode::TabNavigation
                } else {
                    UIMode::Content
                };
            }
        }
    }

    fn render_tabs(
        &self,
        header_area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style =
            (Color::default(), self.selected_tab.palette().c500);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(header_area, buf);
    }

    fn render_footer(
        &self,
        footer_area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        Text::from(vec![
            Line::raw("h / l to change tab | j / k to select diffs/commits |"),
            Line::raw("d to remove a diff | t to truncate | q to quit"),
        ])
        .centered()
        .render(footer_area, buf);
    }
}
