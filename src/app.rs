use std::collections::{HashMap, HashSet};

use ratatui::Frame;
use tokio::sync::mpsc::Sender;

use crate::{
    ai::response::Response,
    config::Config,
    git::{commit::GaiCommit, repo::GaiGit},
    tui::{
        tabs::{SelectedTab, TabContent, TabList},
        ui::UI,
    },
};

pub struct App {
    pub running: bool,
    pub state: State,
    pub cfg: Config,
    pub gai: GaiGit,
    pub ui: UI,

    pub responses: HashMap<String, Result<Response, String>>,
    /// pending ai responses
    pub pending: HashSet<String>,

    /// failed files/hunks
    /// that were NOT RETURNED
    /// by the response
    pub failed_files: Vec<String>,
    pub failed_hunks: Vec<String>,
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
        let state = if cfg.skip_splash {
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
            responses: HashMap::new(),
            pending: HashSet::new(),
            failed_files: Vec::new(),
            failed_hunks: Vec::new(),
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {
        let tab_list = &self.get_list();
        let tab_content = &self.get_content();

        self.ui.render(frame, tab_content, tab_list);
    }

    pub async fn send_request(
        &mut self,
        tx: Sender<(String, Result<Response, String>)>,
    ) {
        let ai = &self.cfg.ai;

        if ai.openai.enable {
            self.pending
                .insert(format!("OpenAI({})", ai.openai.model_name));
        }
        if ai.claude.enable {
            self.pending
                .insert(format!("Claude({})", ai.claude.model_name));
        }
        if ai.gemini.enable {
            self.pending
                .insert(format!("Gemini({})", ai.gemini.model_name));
        }

        let mut diffs = String::new();
        for (file, diff) in &self.gai.get_file_diffs_as_str() {
            diffs.push_str(&format!("File:{}\n{}\n", file, diff));
        }

        let mut rx = ai
            .get_responses(&diffs, self.cfg.stage_hunks)
            .await
            .unwrap();

        tokio::spawn(async move {
            while let Some(from_the_ai) = rx.recv().await {
                let _ = tx.send(from_the_ai).await;
            }
        });
    }

    pub fn apply_commits(&self) {
        match self.ui.selected_tab {
            SelectedTab::Diffs => {}
            SelectedTab::OpenAI
            | SelectedTab::Claude
            | SelectedTab::Gemini => {
                let provider = match self.ui.selected_tab {
                    SelectedTab::OpenAI => "OpenAI",
                    SelectedTab::Claude => "Claude",
                    SelectedTab::Gemini => "Gemini",
                    _ => return,
                };
                let commits: Vec<GaiCommit> = self
                    .responses
                    .iter()
                    .find(|(key, _)| key.starts_with(provider))
                    .and_then(|(_, result)| result.as_ref().ok())
                    .map(|response| {
                        let mut commits = Vec::new();
                        for response_commit in
                            response.commits.to_owned()
                        {
                            commits.push(GaiCommit::from_response(
                                &response_commit,
                                self.gai.capitalize_prefix,
                                self.gai.include_scope,
                            ));
                        }

                        commits
                    })
                    .unwrap_or_default();

                self.gai.apply_commits(&commits);
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

            SelectedTab::OpenAI
            | SelectedTab::Claude
            | SelectedTab::Gemini => {
                let provider = match self.ui.selected_tab {
                    SelectedTab::OpenAI => "OpenAI",
                    SelectedTab::Claude => "Claude",
                    SelectedTab::Gemini => "Gemini",
                    _ => {
                        return TabList {
                            main: Vec::new(),
                            secondary: None,
                            main_title: "Commits".to_owned(),
                            secondary_title: None,
                        };
                    }
                };

                // for now use an empty vec
                // to display failed/no responses
                let main = self
                    .responses
                    .iter()
                    .find(|(key, _)| key.starts_with(provider))
                    .and_then(|(_, result)| result.as_ref().ok())
                    .map(|response| {
                        response
                            .commits
                            .iter()
                            .map(|c| {
                                c.get_commit_prefix(
                                    self.cfg.ai.capitalize_prefix,
                                    self.cfg.ai.include_scope,
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // todo: impl failed
                let (secondary, secondary_title) =
                    if self.cfg.stage_hunks {
                        if self.failed_hunks.is_empty() {
                            (None, None)
                        } else {
                            (
                                Some(self.failed_hunks.clone()),
                                Some("Failed Hunks".to_owned()),
                            )
                        }
                    } else if self.failed_files.is_empty() {
                        (None, None)
                    } else {
                        (
                            Some(self.failed_files.clone()),
                            Some("Failed Hunks".to_owned()),
                        )
                    };

                TabList {
                    main,
                    secondary,
                    main_title: "Commits".to_owned(),
                    secondary_title,
                }
            }
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
                .unwrap_or_else(|| {
                    TabContent::Description(
                        "Select a file to view it's diffs".to_owned(),
                    )
                }),
            SelectedTab::OpenAI
            | SelectedTab::Claude
            | SelectedTab::Gemini => {
                let (provider, enabled) = match selected_tab {
                    SelectedTab::OpenAI => {
                        ("OpenAI", self.cfg.ai.openai.enable)
                    }
                    SelectedTab::Claude => {
                        ("Claude", self.cfg.ai.claude.enable)
                    }
                    SelectedTab::Gemini => {
                        ("Gemini", self.cfg.ai.gemini.enable)
                    }
                    _ => {
                        return TabContent::Description(
                            "No matching AI provider (shouldn't see this btw)".to_owned(),
                        );
                    }
                };

                if !enabled {
                    return TabContent::Description(format!(
                        "{} Provider not enabled",
                        provider
                    ));
                }

                match self
                    .responses
                    .iter()
                    .find(|(key, _)| key.starts_with(provider))
                {
                    Some((_, Ok(response))) => {
                        if let Some(selected) = selected_state_idx
                            && selected < response.commits.len()
                        {
                            let response_commit =
                                &response.commits[selected];

                            let mut content = String::new();
                            content.push_str("files to stage:\n");
                            for file in &response_commit.files {
                                content
                                    .push_str(&format!("{}\n", file));
                            }
                            content.push_str(&format!(
                                "description:\n{}\n",
                                response_commit.message.description
                            ));
                            TabContent::Description(content)
                        } else {
                            TabContent::Description(
                                "Select Commit to View Description/Details".to_owned()
                            )
                        }
                    }
                    Some((_, Err(e))) => TabContent::Description(
                        format!("Error from provider:\n{}", e),
                    ),
                    None => {
                        if self
                            .pending
                            .iter()
                            .any(|p| p.starts_with(provider))
                        {
                            TabContent::Description(
                                "Loading...".to_owned(),
                            )
                        } else {
                            TabContent::Description(
                                "Press p to send a request"
                                    .to_owned(),
                            )
                        }
                    }
                }
            }
        }
    }
}
