pub mod config;
pub mod draw;
pub mod git;
pub mod provider;
pub mod request;
pub mod utils;

use std::{env, error::Error, fs, path::Path};

use dotenv::dotenv;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let cfg = config::Config::init("config.toml")?;

    let mut git_state = git::GitState::new(Path::new("."))?;
    git_state.status(&cfg.files_to_ignore)?;

    let mut diffs = Vec::new();
    for (path, status) in git_state.file {
        println!("Path: {}, Status: {:?}", path, status);
        let diff = fs::read_to_string(path)?;
        diffs.push(diff);
    }

    let api_key = env::var("OPENAI").expect("no env var found");

    let ai = cfg.ai;

    let rb = ai.build_request(&diffs);
    println!("rb: {:?}", rb);

    if cfg.auto_request {
        let recv = ureq::post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", api_key))
            .send_json(&rb)?
            .body_mut()
            .read_to_string();

        println!("recv: {:?}", recv);
    }

    let mut state = draw::App::default();
    let terminal = ratatui::init();
    let result = draw::run(terminal, &mut state);

    ratatui::restore();

    result
}
