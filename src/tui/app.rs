use ratatui::Frame;
use tokio::sync::mpsc;

use crate::{
    ai::response::{Response, get_response},
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

    pub async fn send_request(&mut self, tx: mpsc::Sender<Response>) {
        let ai = &self.cfg.ai;
        let provider = ai.provider;
        let provider_cfg = ai
            .providers
            .get(&provider)
            .expect("somehow did not find provider config")
            .clone();

        let diffs =
            build_diffs_string(self.gai.get_file_diffs_as_str());
        let mut prompt = build_prompt(&self.cfg);

        if ai.include_file_tree {
            prompt.push_str(&self.gai.get_repo_tree());
        }

        tokio::spawn(async move {
            let resp =
                get_response(&diffs, &prompt, provider, provider_cfg)
                    .await;
            let _ = tx.send(resp).await;
        });
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
        match self.ui.selected_tab {
            SelectedTab::Diffs => {
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

                let (secondary, secondary_title) = if secondary
                    .is_empty()
                {
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

            SelectedTab::Commits => {
                if let Some(resp) = &self.response
                    && resp.result.is_ok()
                {
                    let commit_cfg = &self.cfg.gai.commit_config;
                    // kinda jank,
                    // but guaranteed to not be
                    // err
                    let res = resp.result.clone().unwrap();
                    let main: Vec<String> = res
                        .commits
                        .iter()
                        .map(|c| {
                            c.get_commit_prefix(
                                commit_cfg.capitalize_prefix,
                                commit_cfg.include_scope,
                            )
                        })
                        .collect();

                    TabList {
                        main,
                        secondary: None,
                        main_title: "Commits".to_owned(),
                        secondary_title: None,
                    }
                } else {
                    TabList {
                        main: Vec::new(),
                        secondary: None,
                        main_title: String::new(),
                        secondary_title: None,
                    }
                }
            }

            _ => TabList {
                main: Vec::new(),
                secondary: None,
                main_title: String::new(),
                secondary_title: None,
            },
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
            SelectedTab::Commits => {
                if let Some(resp) = &self.response {
                    let res = match &resp.result {
                        Ok(r) => r,
                        Err(e) => {
                            return TabContent::Description(
                                e.to_owned(),
                            );
                        }
                    };

                    if let Some(selected) = selected_state_idx
                        && selected < res.commits.len()
                    {
                        let commit = &res.commits[selected];
                        let mut content = String::new();

                        if self.cfg.gai.stage_hunks {
                            content.push_str("Hunks to Stage:\n");
                            for file in &commit.hunk_ids {
                                content
                                    .push_str(&format!("{} ", file));
                            }
                        } else {
                            content.push_str("Files to Stage:\n");
                            for file in &commit.files {
                                content
                                    .push_str(&format!("{} ", file));
                            }
                        }

                        content.push('\n');
                        content.push_str("Commit Message:\n");
                        content.push_str("Prefix Type: ");
                        content.push_str(
                            format!("{:?}", commit.message.prefix)
                                .as_str(),
                        );
                        content.push('\n');

                        content.push_str("Scope: ");
                        content.push_str(&commit.message.scope);
                        content.push('\n');

                        if commit.message.breaking {
                            content
                                .push_str("Is Breaking Change: Yes");
                        } else {
                            content
                                .push_str("Is Breaking Change: No");
                        }
                        content.push('\n');

                        content.push_str("Header:\n");
                        content.push_str(&commit.message.header);
                        content.push('\n');

                        content.push_str("Body:\n");
                        content.push_str(&commit.message.body);
                        content.push('\n');

                        return TabContent::Description(content);
                    }

                    TabContent::Description(
                        "Select a Commit to View".to_owned(),
                    )
                } else {
                    let model = self
                        .cfg
                        .ai
                        .providers
                        .get(&self.cfg.ai.provider)
                        .expect(
                            "somehow failed to find provider config",
                        )
                        .model
                        .to_owned();

                    TabContent::Description(format!(
                        "Press 'p' to send a request to {}",
                        model
                    ))
                }
            }

            _ => TabContent::Description(
                "Not Yet Implemented".to_owned(),
            ),
        }
    }
}
