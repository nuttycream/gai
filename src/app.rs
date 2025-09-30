use anyhow::{Result, bail};
use std::collections::HashMap;

use crate::{config::Config, git::GaiGit, response::Response};

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

impl App {
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

    pub async fn send_request(&mut self) -> Result<Response> {
        let ai = &self.cfg.ai;

        let mut diffs = String::new();
        for (file, diff) in &self.diffs {
            diffs.push_str(&format!("File:{}\n{}\n", file, diff));
        }
        let extractors = ai.build_requests()?;

        if let Some(gemini) = extractors.gemini {
            match gemini.extract(&diffs).await {
                Ok(response) => return Ok(response),
                Err(e) => eprintln!("Gemini failed: {}", e),
            }
        }

        if let Some(openai) = extractors.openai {
            match openai.extract(&diffs).await {
                Ok(response) => return Ok(response),
                Err(e) => eprintln!("OpenAI failed: {}", e),
            }
        }

        if let Some(claude) = extractors.claude {
            match claude.extract(&diffs).await {
                Ok(response) => return Ok(response),
                Err(e) => eprintln!("Claude failed: {}", e),
            }
        }

        bail!("No AI providers enabled or all failed");
    }

    pub fn apply_ops(&self, response: &Response) {
        self.gai.apply_commits(&response.commits);
    }
}
