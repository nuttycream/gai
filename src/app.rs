use std::collections::HashMap;

use tokio::sync::mpsc::Sender;

use crate::{
    config::Config, git::GaiGit, response::Response,
    tabs::SelectedTab,
};

pub struct App {
    pub running: bool,
    pub state: State,
    pub cfg: Config,
    pub gai: GaiGit,
    pub responses: HashMap<String, Result<Response, String>>,
}

pub enum State {
    /// initializing gai:
    /// checks for existing repo
    /// does a diff check
    /// and gathers the data
    /// for the user to send
    Splash,

    /// state where gai is sending
    /// a request or waiting to
    /// receive the response.
    /// This is usually one continous
    /// moment.
    SendingRequest(Sender<Response>),

    /// state where the user can
    /// see what to send
    /// to the AI provider
    DiffView,

    /// response view
    OpsView(Response),
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
            State::DiffView
        } else {
            State::Splash
        };

        Self {
            running: true,
            state,
            cfg,
            gai,
            responses: HashMap::new(),
        }
    }

    pub async fn send_request(
        &mut self,
        tx: Sender<(String, Result<Response, String>)>,
    ) {
        let ai = &self.cfg.ai;

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

    pub fn apply_ops(&self, response: &Response) {
        self.gai.apply_commits(&response.commits);
    }

    pub fn get_list(&self, selected_tab: SelectedTab) -> Vec<String> {
        match selected_tab {
            SelectedTab::Diffs => {
                self.gai.diffs.clone().into_keys().collect()
            }
            SelectedTab::OpenAI
            | SelectedTab::Claude
            | SelectedTab::Gemini => {
                let provider = match selected_tab {
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

    pub fn get_content(
        &self,
        selected_tab: SelectedTab,
        selection_list: &[String],
        selected_state_idx: Option<usize>,
    ) -> String {
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
                let provider = match selected_tab {
                    SelectedTab::OpenAI => "OpenAI",
                    SelectedTab::Claude => "Claude",
                    SelectedTab::Gemini => "Gemini",
                    _ => return String::new(),
                };

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
                        format!("error from provider: {}", e)
                    }
                    None => "p to send request".to_owned(),
                }
            }
        }
    }
}
