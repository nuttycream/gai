pub mod app;
pub mod config;
pub mod draw;
pub mod git;
pub mod provider;
pub mod request;
pub mod response;
pub mod utils;

use std::{collections::HashMap, error::Error, path::Path};

use dotenv::dotenv;

use crate::{
    draw::UI,
    git::{diff::GitDiff, ops::GitOps},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let cfg = config::Config::init("config.toml")?;

    let mut git_state = git::state::GitState::new(Path::new("."))?;
    git_state.status(&cfg.files_to_ignore)?;

    let mut git_diff = GitDiff::new();

    // temp
    let _ = git_diff.create_diffs(&git_state.repo);

    // temp not using actual val of create_diffs
    // todo: put this in draw.rs
    // we need to give color to the diffs
    // in the diffview
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

    let mut state = crate::app::App::default();
    state.init(cfg);
    state.load_diffs(diffs);
    let ops = state.send_request().await?.ops;
    println!("{:#?}", ops);

    //let op = GitOps::init(ops, &git_state.repo);
    //op.apply_ops();

    //state.load_recv(&recv);
    /* let terminal = ratatui::init();
    let mut ui = UI::default();
    let result = ui.run(terminal, &mut state);

    ratatui::restore();

    result */
    Ok(())
}
