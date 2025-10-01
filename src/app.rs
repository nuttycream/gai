use crate::{config::Config, git::GaiGit, response::Response};

pub struct App {
    pub running: bool,
    pub state: State,
    pub cfg: Config,
    pub gai: GaiGit,
    pub ops: Option<Vec<Response>>,
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
    Pending,

    /// state where the user can
    /// see what to send
    /// to the AI provider
    DiffView { selected: usize },

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
}

impl App {
    pub fn new(cfg: Config, gai: GaiGit) -> Self {
        let state = if cfg.skip_splash {
            State::DiffView { selected: 0 }
        } else {
            State::Splash
        };

        Self {
            running: true,
            state,
            cfg,
            gai,
            ops: None,
        }
    }

    pub fn get_diff_content(&self, path: &str) {}

    pub async fn send_request(&mut self) {
        let ai = &self.cfg.ai;

        let mut diffs = String::new();
        for (file, diff) in &self.gai.diffs {
            diffs.push_str(&format!("File:{}\n{}\n", file, diff));
        }

        let mut rx = ai.get_responses(&diffs).await.unwrap();

        while let Some((provider, resp)) = rx.recv().await {
            match resp {
                Ok(resp) => {
                    println!("{}\n{:#?}", provider, resp);
                }
                Err(e) => println!("failed: {e}"),
            }
        }

        // ai.get_responses(&diffs).await
    }

    pub fn apply_ops(&self, response: &Response) {
        self.gai.apply_commits(&response.commits);
    }
}
