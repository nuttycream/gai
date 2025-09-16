pub mod config;
pub mod draw;
pub mod git;

use std::{fs, path::Path};

use crate::{config::Config, git::GitState};
use color_eyre::{Result, eyre::Ok};

fn main() -> Result<()> {
    color_eyre::install()?;

    let cfg = Config::init();

    let mut git_state = GitState::new(Path::new("."))?;
    git_state.status(&cfg.ignore_config.files_to_ignore)?;

    // let terminal = ratatui::init();
    //let result = run(terminal, &mut state);

    //ratatui::restore();

    for (path, status) in git_state.file {
        println!("Path: {}, Status: {:?}", path, status);
        let file_string = fs::read_to_string(path)?;
        println!("file_string: {}", file_string);
    }

    Ok(())
    //result
}
