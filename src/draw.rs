use ratatui::{
    Frame,
    widgets::{ListState, ScrollbarState},
};

use crate::app::App;

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
}

impl UI {
    pub fn render(&mut self, frame: &mut Frame, app: &App) {}

    pub fn scroll_up(&mut self, app: &App) {
        if self.in_content_mode {
            self.content_scroll =
                self.content_scroll.saturating_sub(1);
            //self.update_content_scroll();
        } else {
            self.file_path_state.select_previous();
            // self.update_curr_diff(app_state);
            // self.update_file_scroll();
        }
    }

    pub fn scroll_down(&mut self, app: &App) {
        if self.in_content_mode {
            self.content_scroll =
                self.content_scroll.saturating_add(1);
            //self.update_content_scroll();
        } else {
            self.file_path_state.select_next();
            //self.update_curr_diff(app_state);
            //self.update_file_scroll();
        }
    }

    pub fn focus_left(&mut self, app: &App) {
        self.in_content_mode = false;
    }

    pub fn focus_right(&mut self, app: &App) {
        self.in_content_mode = true;
    }

    pub fn select_item(&mut self, app: &App) -> Option<usize> {
        None
    }
}
