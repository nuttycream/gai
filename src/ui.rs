use ratatui::{Frame, widgets::ListState};

use crate::app::App;

#[derive(Default)]
pub struct UI {
    pub selection_list: Vec<String>,
    pub selected_state: ListState,

    pub in_content_mode: bool,
    pub content_scroll: u16,
    pub content_text: String,
}

impl UI {
    pub fn new() -> Self {
        let mut selected_state = ListState::default();
        selected_state.select_first();

        Self {
            selection_list: Vec::new(),
            selected_state,

            in_content_mode: false,
            content_scroll: 0,
            content_text: String::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, app: &App) {
        match &app.state {
            crate::app::State::Splash => self.draw_splash(frame),

            crate::app::State::SendingRequest(_) => {
                self.draw_pending(frame)
            }

            crate::app::State::DiffView => {
                self.selection_list =
                    app.gai.diffs.clone().into_keys().collect();

                if let Some(selected) = self.selected_state.selected()
                    && selected < app.gai.diffs.len()
                    && let Some(diff) = app
                        .gai
                        .diffs
                        .get(&self.selection_list[selected])
                {
                    self.content_text = diff.to_owned();
                }

                self.draw_diffview(frame)
            }

            crate::app::State::OpsView { .. } => {
                self.draw_opsview(frame, app.ops.as_deref());
            }
        }
    }

    pub fn scroll_up(&mut self, app: &App) {
        if self.in_content_mode {
            self.content_scroll =
                self.content_scroll.saturating_sub(1);
        } else {
            self.selected_state.select_previous();
        }
    }

    pub fn scroll_down(&mut self, app: &App) {
        if self.in_content_mode {
            self.content_scroll =
                self.content_scroll.saturating_add(1);
        } else {
            self.selected_state.select_next();
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
