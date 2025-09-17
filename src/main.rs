pub mod config;
pub mod draw;
pub mod git;
pub mod request;
pub mod utils;

use std::{env, error::Error, path::Path};

use crate::{
    config::Config,
    draw::{App, run},
    git::GitState,
    request::{InputData, RequestBody},
};
use dotenv::dotenv;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let cfg = Config::init("config.toml")?;

    let mut git_state = GitState::new(Path::new("."))?;
    git_state.status(&cfg.ignore_config.files_to_ignore)?;

    for (path, status) in git_state.file {
        println!("Path: {}, Status: {:?}", path, status);
    }

    let mut rb = RequestBody::new();
    let mut input = InputData::new();
    input.add_data("test")?;
    rb.add_input(input)?;

    let recv = ureq::post("https://api.openai.com/v1/responses")
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                env::var("OPENAI").expect("no env var found")
            ),
        );

    println!("recv: {:?}", recv);
    println!("rb: {:?}", rb);

    let mut state = App::default();
    let terminal = ratatui::init();
    let result = run(terminal, &mut state);

    ratatui::restore();

    result
}
