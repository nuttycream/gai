use std::collections::{HashMap, HashSet};

use ratatui::Frame;
use tokio::sync::mpsc::Sender;

use crate::{
    config::Config,
    git::GaiGit,
    response::{Commit, Response},
    tabs::SelectedTab,
    ui::UI,
};

pub struct App {
    pub running: bool,
    pub state: State,
    pub cfg: Config,
    pub gai: GaiGit,
    pub ui: UI,

    pub responses: HashMap<String, Result<Response, String>>,
    pub pending: HashSet<String>,
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
    RemoveCurrentItem,

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
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {
        let items = &self.get_list();
        let content = &self.get_content();

        self.ui.render(frame, items, content);
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
        for (file, diff) in &self.gai.diffs {
            diffs.push_str(&format!("File:{}\n{}\n", file, diff));
        }

        let mut rx = ai.get_responses(&diffs).await.unwrap();
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
                let commits: Vec<Commit> = self
                    .responses
                    .iter()
                    .find(|(key, _)| key.starts_with(provider))
                    .and_then(|(_, result)| result.as_ref().ok())
                    .map(|response| response.commits.to_owned())
                    .unwrap_or_default();

                self.gai.apply_commits(&commits);
            }
        }
    }

    pub fn get_list(&self) -> Vec<String> {
        match self.ui.selected_tab {
            SelectedTab::Diffs => {
                self.gai.diffs.clone().into_keys().collect()
            }
            SelectedTab::OpenAI
            | SelectedTab::Claude
            | SelectedTab::Gemini => {
                let provider = match self.ui.selected_tab {
                    SelectedTab::OpenAI => "OpenAI",
                    SelectedTab::Claude => "Claude",
                    SelectedTab::Gemini => "Gemini",
                    _ => return Vec::new(),
                };

                // for now use an empty vec
                // to display failed/no responses
                self.responses
                    .iter()
                    .find(|(key, _)| key.starts_with(provider))
                    .and_then(|(_, result)| result.as_ref().ok())
                    .map(|response| {
                        response
                            .commits
                            .iter()
                            .map(|c| c.get_commit_prefix(&self.cfg))
                            .collect()
                    })
                    .unwrap_or_default()
            }
        }
    }

    pub fn get_content(&self) -> String {
        let selection_list = self.get_list();
        let selected_tab = self.ui.selected_tab;
        let selected_state_idx = self.ui.selected_state.selected();

        match selected_tab {
            SelectedTab::Diffs => {
                if let Some(selected) = selected_state_idx
                    && selected < self.gai.diffs.len()
                    && let Some(diff) =
                        self.gai.diffs.get(&selection_list[selected])
                {
                    diff.to_owned()
                } else {
                    "select a file to view it's diff".to_owned()
                }
            }
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
                    _ => return String::new(),
                };

                if !enabled {
                    return "Not Enabled".to_owned();
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
                            let commit = &response.commits[selected];

                            let mut content = String::new();
                            content.push_str("files to stage:\n");
                            for file in &commit.files {
                                content
                                    .push_str(&format!("{}\n", file));
                            }
                            content.push_str(&format!(
                                "description:\n{}\n",
                                commit.get_commit_message(&self.cfg)
                            ));
                            content
                        } else {
                            "select commit to view details".to_owned()
                        }
                    }
                    Some((_, Err(e))) => {
                        format!("Error from provider:\n{}", e)
                    }
                    None => {
                        if self
                            .pending
                            .iter()
                            .any(|p| p.starts_with(provider))
                        {
                            "Loading...".to_owned()
                        } else {
                            "Press p to send a request".to_owned()
                        }
                    }
                }
            }
        }
    }
}
