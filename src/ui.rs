use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Color,
    text::Line,
    widgets::{ListState, Tabs, Widget},
};
use strum::IntoEnumIterator;

use crate::tabs::{SelectedTab, TabContent};

#[derive(Default)]
pub struct UI {
    pub selected_tab: SelectedTab,

    pub selected_state: ListState,
}

// todo, implement this
// use vertical keys to select
// tab -> content
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
        }
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        items: &[String],
        tab_content: &TabContent,
    ) {
        use Constraint::{Length, Min};
        let vertical =
            Layout::vertical([Length(1), Min(0), Length(1)])
                .margin(5);
        let [header_area, inner_area, footer_area] =
            vertical.areas(frame.area());

        self.render_tabs(header_area, frame.buffer_mut());

        self.selected_tab.render(
            inner_area,
            frame.buffer_mut(),
            items,
            tab_content,
            &mut self.selected_state,
        );

        self.render_footer(footer_area, frame.buffer_mut());
    }

    pub fn scroll_up(&mut self) {
        self.selected_state.select_previous();
    }

    pub fn scroll_down(&mut self) {
        self.selected_state.select_next();
    }

    pub fn focus_left(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    pub fn focus_right(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn goto_tab(&mut self, tab: usize) {
        self.selected_tab =
            self.selected_tab.find_tab(tab.saturating_sub(1));
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
        Line::raw(
            "h / l to change tab | j / k to select diffs/commits | \
            d to remove a diff | t to toggle truncation | q to quit",
        )
        .centered()
        .render(footer_area, buf);
    }
}
