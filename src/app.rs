use anyhow::Result;
use std::collections::HashMap;

use crate::{
    config::Config,
    git::GaiGit,
    response::{Commit, Response},
};

pub struct App {
    pub state: State,
    pub cfg: Config,
    pub gai: GaiGit,
    pub diffs: HashMap<String, String>,
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
enum Action {
    SelectFile(usize),
    SelectCommit(usize),
    ScrollUp,
    ScrollDown,
    SendRequest,
}

/// only specific to gai
/// like sending/recieving requests
enum GaiCommand {
    SendRequest(String),
    ApplyCommits(Vec<Commit>),
}

impl App {
    pub fn update(
        &mut self,
        action: Action,
    ) -> Result<Option<GaiCommand>> {
        match action {
            Action::SelectFile(file_idx) => todo!(),
            Action::SelectCommit(commit_idx) => todo!(),
            Action::ScrollUp => todo!(),
            Action::ScrollDown => todo!(),
            Action::SendRequest => todo!(),
            //_ => {}
        }
    }

    pub fn switch_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    pub fn get_file_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> =
            self.diffs.keys().cloned().collect();
        paths.sort();
        paths
    }

    pub fn get_diff_content(&self, path: &str) -> String {
        self.diffs
            .get(path)
            .cloned()
            .unwrap_or_else(|| String::from("no diff found"))
    }

    pub async fn send_request(&mut self) {
        let ai = &self.cfg.ai;

        let mut diffs = String::new();
        for (file, diff) in &self.diffs {
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
