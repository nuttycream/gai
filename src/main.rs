pub mod app;
pub mod config;
pub mod draw;
pub mod git;
pub mod provider;
pub mod request;
pub mod response;
pub mod utils;

use std::{collections::HashMap, env, error::Error, fs, path::Path};

use dotenv::dotenv;

use crate::{draw::UI, git::diff::GitDiff, response::Response};

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let cfg = config::Config::init("config.toml")?;

    let mut git_state = git::state::GitState::new(Path::new("."))?;
    git_state.status(&cfg.files_to_ignore)?;

    let mut git_diff = GitDiff::new();

    // temp
    let _ = git_diff.create_diffs(&git_state.repo);

    // temp not using actual val of create_diffs
    let mut diffs = HashMap::new();
    for (path, _status) in &git_state.file {
        if let Some(hunks) = git_diff.diffs.get(path) {
            let mut diff_str = String::new();

            for hunk in hunks {
                diff_str.push_str(&hunk.header);
                diff_str.push('\n');

                for line in &hunk.line_diffs {
                    let prefix = match line.diff_type {
                        git::diff::DiffType::Unchanged => ' ',
                        git::diff::DiffType::Additions => '+',
                        git::diff::DiffType::Deletions => '-',
                    };
                    diff_str.push(prefix);
                    diff_str.push_str(&line.content);
                }
                diff_str.push('\n');
            }
            if !diff_str.trim().is_empty() {
                diffs.insert(path.clone(), diff_str);
            }
        }
    }

    let api_key = env::var("OPENAI").expect("no env var found");

    let ai = &cfg.ai;

    let rb = ai.build_request(diffs.to_owned());
    //println!("rb: {:#?}", rb);

    let mut recv = String::new();
    if cfg.auto_request {
        recv = ureq::post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", api_key))
            .send_json(&rb)?
            .body_mut()
            .read_to_string()?;

        //println!("recv: {:#?}", recv);
    }

    let jason: serde_json::Value = serde_json::from_str(&recv)?;

    let resp_str = jason["output"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "message")
        .unwrap()["content"][0]["text"]
        .as_str()
        .unwrap();

    let resp: Response = serde_json::from_str(resp_str)?;

    println!("{:#?}", resp);

    let mut state = crate::app::App::default();
    let terminal = ratatui::init();
    state.init(&cfg);
    state.load_diffs(diffs);
    state.load_recv(&recv);
    let mut ui = UI::default();
    let result = ui.run(terminal, &mut state);

    ratatui::restore();

    result
}
