pub mod draw;
pub mod git;

use std::path::Path;

use crate::git::GitState;
use color_eyre::{Result, eyre::Ok};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut git_state = GitState::new(Path::new("."))?;
    git_state.status()?;

    // let terminal = ratatui::init();
    //let result = run(terminal, &mut state);

    //ratatui::restore();

    for (path, status) in git_state.file {
        println!("Path: {}, Status: {:?}", path, status);
    }

    Ok(())
    //result
}
