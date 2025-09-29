pub mod app;
pub mod config;
pub mod draw;
pub mod git;
pub mod provider;
pub mod response;
pub mod utils;

use std::{collections::HashMap, error::Error, fs, path::Path};

use dotenv::dotenv;

use crate::{
    draw::UI,
    git::{DiffType, Status},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let cfg = config::Config::init("config.toml")?;

    let mut gai = git::GaiGit::new(Path::new("."))?;

    // todo remove, we don't really need to track the
    // state no? or should we keep it.
    gai.status(&cfg.files_to_ignore)?;

    gai.create_diffs().unwrap();

    // temp not using actual val of create_diffs
    // todo: put this in draw.rs
    // we need to give color to the diffs
    // in the diffview
    let mut diffs = HashMap::new();
    for (path, status) in &gai.file {
        match status {
            Status::Changed(changed) => {}
            Status::Untracked => {
                if cfg.include_untracked {
                    diffs.insert(
                        path.to_owned(),
                        fs::read_to_string(path).unwrap(),
                    );
                }
            }
        }
        if let Some(hunks) = gai.diffs.get(path) {
            let mut diff_str = String::new();

            for hunk in hunks {
                diff_str.push_str(&hunk.header);
                diff_str.push('\n');

                for line in &hunk.line_diffs {
                    let prefix = match line.diff_type {
                        DiffType::Unchanged => ' ',
                        DiffType::Additions => '+',
                        DiffType::Deletions => '-',
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

    let mut state = crate::app::App {
        state: app::State::Splash,
        cfg,
        diffs,
        gai,
    };

    let terminal = ratatui::init();
    let mut ui = UI::default();
    let result = ui.run(terminal, &mut state).await;

    ratatui::restore();

    result

    //Ok(())
}
