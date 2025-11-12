use anyhow::Result;
use dialoguer::{Password, theme::ColorfulTheme};
use std::{fs, path::PathBuf};

use crate::create_spinner_bar;

pub fn auth_login() -> Result<()> {
    println!("Opening Browser for https://cli.gai.fyi/login");
    open::that("https://cli.gai.fyi/login")?;
    let token = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Paste Token: ")
        .interact()?;

    println!("Storing token of length: {}", token.len());

    store_token(&token)?;
    Ok(())
}

pub async fn auth_status() -> Result<()> {
    let bar = create_spinner_bar();

    bar.set_message("Grabbing Status");
    let token = get_token()?;

    let client = reqwest::Client::new();
    let resp = client
        .get("https://cli.gai.fyi/status")
        .bearer_auth(token)
        .send()
        .await?;

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Status {
        requests_made: i32,
        expiration: u64,
    }

    let status = resp.json::<Status>().await?;

    bar.finish();
    bar.reset();

    if let Some(date) = chrono::DateTime::from_timestamp(
        status.expiration.try_into()?,
        0,
    ) {
        println!("Requests made: {}/10", status.requests_made);
        println!("Resets at {}", date);
    } else {
        println!("Failed to convert expiration to datetime");
    }

    Ok(())
}

pub fn clear_auth() -> Result<()> {
    let token_path = token_path()?;

    if token_path.exists() {
        fs::remove_file(token_path)?;
    }
    println!("No longer aunthenticated");
    Ok(())
}

pub fn get_token() -> Result<String> {
    let token_path = token_path()?;

    Ok(fs::read_to_string(token_path)?.trim().to_string())
}

fn store_token(token: &str) -> Result<()> {
    if token.is_empty() {
        return Err(anyhow::anyhow!("token cannot be empty"));
    }

    let token_path = token_path()?;

    fs::write(&token_path, token)?;

    Ok(())
}

fn token_path() -> Result<PathBuf> {
    let cfg_dir =
        directories::ProjectDirs::from("com", "nuttycream", "gai")
            .ok_or_else(|| {
                anyhow::anyhow!("Can't find the config directory")
            })?;

    Ok(cfg_dir.config_dir().join(".token"))
}
