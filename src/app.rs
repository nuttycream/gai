use std::{collections::HashMap, env, error::Error};

use crate::{config::Config, response::Response};

#[derive(Default)]
pub struct App {
    pub state: State,

    pub cfg: Config,

    pub diffs: HashMap<String, String>,
}

#[derive(Default)]
pub enum State {
    /// initializing gai:
    /// checks for existing repo
    /// does a diff check
    /// and gathers the data
    /// for the user to send
    #[default]
    Warmup,

    /// state where gai is sending
    /// a request or waiting to
    /// receive the response.
    /// This is usually one continous
    /// moment.
    Pending,

    /// state where the user can
    /// either: see what to send
    /// to the AI provider
    /// or what the AI provider has
    /// sent back
    Running,
}

impl App {
    pub fn init(&mut self, cfg: Config) {
        self.cfg = cfg;
    }

    pub fn switch_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    pub fn load_diffs(&mut self, files: HashMap<String, String>) {
        self.diffs = files.to_owned();
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

    pub fn send_request(&self) -> Result<Response, Box<dyn Error>> {
        let api_key = env::var("OPENAI").expect("no env var found");

        let ai = &self.cfg.ai;

        let rb = ai.build_request(self.diffs.to_owned());
        //println!("rb: {:#?}", rb);

        let recv = ureq::post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", api_key))
            .send_json(&rb)?
            .body_mut()
            .read_to_string()?;

        //println!("recv: {:#?}", recv);

        match serde_json::from_str::<serde_json::Value>(&recv) {
            Ok(jason) => {
                let resp_str = jason["output"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|item| item["type"] == "message")
                    .unwrap()["content"][0]["text"]
                    .as_str()
                    .unwrap();

                //println!("{:#?}", resp.ops);
                return Ok(Response::new(resp_str));
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
}
