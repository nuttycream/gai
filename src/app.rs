use std::collections::{HashMap, HashSet};

use ratatui::Frame;
use tokio::sync::mpsc::Sender;

use crate::{
    ai::response::{GaiCommit, Response},
    config::Config,
    git::repo::GaiGit,
    tui::tabs::{SelectedTab, TabContent},
    tui::ui::UI,
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
        }
    }

    pub fn run(&mut self, frame: &mut Frame) {
        let items = &self.get_list();
        let tab_content = &self.get_content();

        self.ui.render(frame, items, tab_content);
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
                let commits: Vec<GaiCommit> = self
                    .responses
                    .iter()
                    .find(|(key, _)| key.starts_with(provider))
                    .and_then(|(_, result)| result.as_ref().ok())
                    .map(|response| response.commits.to_owned())
                    .unwrap_or_default();

                self.gai.apply_commits(&commits, &self.cfg);
            }
        }
    }

    pub fn remove_selected(&mut self) {
        if let SelectedTab::Diffs = self.ui.selected_tab {
            let selection_list = self.get_list();
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

    fn get_list(&self) -> Vec<String> {
        match self.ui.selected_tab {
            SelectedTab::Diffs => self
                .gai
                .files
                .iter()
                .map(|g| g.path.to_owned())
                .collect(),
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

    fn get_content(&self) -> TabContent {
        let selection_list = self.get_list();
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
