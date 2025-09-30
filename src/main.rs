pub mod app;
pub mod config;
pub mod consts;
pub mod draw;
pub mod git;
pub mod provider;
pub mod response;
pub mod utils;

use crate::draw::UI;

use anyhow::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cfg = config::Config::init("config.toml")?;

    let mut gai = git::GaiGit::new(".")?;
    gai.create_diffs(&cfg.files_to_ignore)?;
    let diffs = gai.diffs.to_owned();

    let state = if cfg.skip_splash {
        app::State::DiffView { selected: 0 }
    } else {
        app::State::Splash
    };

    let mut app_state = crate::app::App {
        state,
        cfg,
        diffs,
        gai,
    };
    let mut ui = UI::default();
    let terminal = ratatui::init();

    //
    loop {

        if matches!(app_state.state)

    }

    ratatui::restore();

    Ok(())

}
