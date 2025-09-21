pub mod app;
pub mod config;
pub mod draw;
pub mod git;
pub mod provider;
pub mod request;
pub mod utils;

use std::{collections::HashMap, env, error::Error, fs, path::Path};

use dotenv::dotenv;

use crate::{draw::UI, git::diff::GitDiff};

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let cfg = config::Config::init("config.toml")?;

    let mut git_state = git::state::GitState::new(Path::new("."))?;
    git_state.status(&cfg.files_to_ignore)?;

    let mut diffs = HashMap::new();
    for (path, status) in git_state.file {
        println!("Path: {}, Status: {:?}", path, status);
        let diff = fs::read_to_string(&path)?;
        diffs.insert(path, diff);
    }

    let mut git_diff = GitDiff::new();
    let _ = git_diff.create_diffs(&git_state.repo);

    let api_key = env::var("OPENAI").expect("no env var found");

    let ai = cfg.ai;

    let string_to = diffs.values().cloned().collect::<Vec<String>>();
    let rb = ai.build_request(&string_to);
    //println!("rb: {:?}", rb);

    if cfg.auto_request {
        let recv = ureq::post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", api_key))
            .send_json(&rb)?
            .body_mut()
            .read_to_string();

        println!("recv: {:?}", recv);
    }

    let mut state = crate::app::App::default();
    let terminal = ratatui::init();
    state.load_diffs(diffs);
    let mut ui = UI::default();
    let result = ui.run(terminal, &mut state);

    ratatui::restore();

    result
}
