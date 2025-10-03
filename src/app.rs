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
    pub responses: Option<Vec<Response>>,
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
            responses: None,
        }
    }

    pub async fn switch_state(&mut self, new_state: State) {
        self.state = new_state;

        if let State::SendingRequest(tx) = &self.state {
            self.send_request(tx.clone()).await;
        }
    }

    pub async fn send_request(&mut self, tx: Sender<Response>) {
        let ai = &self.cfg.ai;

        let mut diffs = String::new();
        for (file, diff) in &self.gai.diffs {
            diffs.push_str(&format!("File:{}\n{}\n", file, diff));
        }

        let mut rx = ai.get_responses(&diffs).await.unwrap();

        while let Some((provider, result)) = rx.recv().await {
            match result {
                Ok(resp) => {
                    //println!("{}\n{:#?}", provider, resp);
                    let _ = tx.send(resp).await;
                }
                Err(e) => println!("failed: {e}"),
            }
        }

        // ai.get_responses(&diffs).await
    }

    pub fn apply_ops(&self, response: &Response) {
        self.gai.apply_commits(&response.commits);
    }

    pub fn get_list(&self, selected_tab: SelectedTab) -> Vec<String> {
        match selected_tab {
            SelectedTab::Diffs => {
                self.gai.diffs.clone().into_keys().collect()
            }
            SelectedTab::OpenAI => Vec::new(),
            SelectedTab::Claude => Vec::new(),
            SelectedTab::Gemini => Vec::new(),
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

            SelectedTab::OpenAI => String::new(),
            SelectedTab::Claude => String::new(),
            SelectedTab::Gemini => String::new(),
        }
    }
}
