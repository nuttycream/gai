use ratatui::Frame;

use crate::{
    ai::response::Response,
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    tui::{
        tabs::{SelectedTab, TabContent, TabList},
        ui::UI,
    },
    utils::{build_diffs_string, build_prompt},
};

pub struct App {
    pub running: bool,
    pub state: State,
    pub cfg: Config,
    pub gai: GaiGit,
    pub ui: UI,

    pub response: Option<Response>,
}

pub enum State {
    Running,
    Splash,
}

/// various ui actions
pub enum Action {
    ScrollUp,
    ScrollDown,

    FocusLeft,
    FocusRight,

    NextTab,
    PreviousTab, // shift+tab(?)

    SendRequest,
    ApplyCommits,
    RemoveCurrentSelected,
    TruncateCurrentSelected,

    Quit,

    DiffTab,
    OpenAITab,
    ClaudeTab,
    GeminiTab,
}

impl App {
    pub fn new(cfg: Config, gai: GaiGit) -> Self {
        let state = if cfg.tui.skip_splash {
            State::Running
        } else {
            State::Splash
        };

        Self {
            running: true,
            state,
            cfg,
            gai,
            ui: UI::new(),
            response: None,
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {
        let tab_list = &self.get_list();
        let tab_content = &self.get_content();

        self.ui.render(frame, tab_content, tab_list);
    }

    pub async fn send_request(&mut self) {
        let ai = &self.cfg.ai;

        let diffs =
            build_diffs_string(self.gai.get_file_diffs_as_str());
        let mut prompt = build_prompt(&self.cfg);

        // todo wth am i doing
        if ai.include_file_tree {
            prompt.push_str(&self.gai.get_repo_tree());
        }
    }

    pub fn apply_commits(&self) {
        match self.ui.selected_tab {
            SelectedTab::Diffs => {}
            _ => {
                if let Some(data) = &self.response
                    && data.result.is_ok()
                {
                    let commits: Vec<GaiCommit> = data
                        .result
                        .to_owned()
                        .unwrap()
                        .commits
                        .iter()
                        .map(|response_commit| {
                            GaiCommit::from_response(
                                response_commit,
                                self.gai.capitalize_prefix,
                                self.gai.include_scope,
                            )
                        })
                        .collect();

                    self.gai.apply_commits(&commits);
                }
            }
        }
    }

    pub fn remove_selected(&mut self) {
        if let SelectedTab::Diffs = self.ui.selected_tab {
            let selection_list = self.get_list().main;
            let selected_state_idx =
                self.ui.selected_state.selected();
            if let Some(selected) = selected_state_idx
                && selected < self.gai.files.len()
            {
                let selected_file = &selection_list[selected];
                if let Some(pos) = self
                    .gai
                    .files
                    .iter()
                    .position(|g| g.path == *selected_file)
                {
                    self.gai.files.remove(pos);
                }
            }
        }
    }

    pub fn truncate_selected(&mut self) {
        if let SelectedTab::Diffs = self.ui.selected_tab {
            let selected_state_idx =
                self.ui.selected_state.selected();
            if let Some(selected) = selected_state_idx
                && selected < self.gai.files.len()
            {
                self.gai.files[selected].should_truncate =
                    !self.gai.files[selected].should_truncate;
            }
        }
    }

    fn get_list(&self) -> TabList {
        let main = self
            .gai
            .files
            .iter()
            .filter(|g| !g.should_truncate)
            .map(|g| g.path.to_owned())
            .collect();

        let secondary: Vec<String> = self
            .gai
            .files
            .iter()
            .filter(|g| g.should_truncate)
            .map(|g| g.path.to_owned())
            .collect();

        let (secondary, secondary_title) = if secondary.is_empty() {
            (None, None)
        } else {
            (Some(secondary), Some("Truncated".to_owned()))
        };

        TabList {
            main,
            secondary,
            main_title: "Files".to_owned(),
            secondary_title,
        }
    }

    fn get_content(&self) -> TabContent {
        let selection_list = self.get_list().main;
        let selected_tab = self.ui.selected_tab;
        let selected_state_idx = self.ui.selected_state.selected();

        match selected_tab {
            SelectedTab::Diffs => selected_state_idx
                .filter(|&selected| selected < selection_list.len())
                .and_then(|selected| {
                    self.gai
                        .files
                        .iter()
                        .find(|gai| {
                            gai.path == selection_list[selected]
                        })
                        .map(|gai| {
                            if gai.should_truncate {
                                TabContent::Description(
                                    "Truncated File".to_owned(),
                                )
                            } else {
                                TabContent::Diff(gai.hunks.clone())
                            }
                        })
                })
                .unwrap_or(TabContent::Description(
                    "Select a file to view its diffs".to_owned(),
                )),

            _ => TabContent::Description("test".to_owned()),
        }
    }
}
