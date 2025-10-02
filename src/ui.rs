use ratatui::{
    layout::{Constraint, Layout},
    style::Color,
    text::Line,
    widgets::{Tabs, Widget},
};
use strum::IntoEnumIterator;

use crate::{app::App, tabs::SelectedTab};

#[derive(Default)]
pub struct UI {
    selected_tab: SelectedTab,
}

// todo, implement this
// use vertical keys to select
// tab -> content
#[derive(Default)]
enum UIMode {
    #[default]
    TabNavigation,
    Content,

    // todo, special mode
    // to edit commit message
    // etc, prolly a popup
    Edit,
}

impl Widget for &UI {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) where
        Self: Sized,
    {
        use Constraint::{Length, Min};
        let vertical =
            Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] =
            vertical.areas(area);

        self.render_tabs(header_area, buf);
        self.selected_tab.render(inner_area, buf);
        self.render_footer(footer_area, buf);
    }
}

impl UI {
    pub fn new() -> Self {
        Self {
            selected_tab: SelectedTab::Diffs,
        }
    }

    //pub fn render(&mut self, frame: &mut Frame, app: &App) {}

    pub fn scroll_up(&mut self, app: &App) {}

    pub fn scroll_down(&mut self, app: &App) {}

    pub fn focus_left(&mut self, app: &App) {
        self.selected_tab = self.selected_tab.previous();
    }

    pub fn focus_right(&mut self, app: &App) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn goto_tab(&mut self, tab: usize) {
        self.selected_tab =
            self.selected_tab.find_tab(tab as usize - 1);
    }

    pub fn select_item(&mut self, app: &App) -> Option<usize> {
        None
    }

    fn render_tabs(
        &self,
        header_area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style =
            (Color::default(), self.selected_tab.palette().c700);
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
        Line::raw("h / l to change tab | Press q to quit")
            .centered()
            .render(footer_area, buf);
    }
}
