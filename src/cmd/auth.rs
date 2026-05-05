use anyhow::Result;
use std::{fs, io::ErrorKind, path::PathBuf};

use crate::args::Auth;

pub fn run(auth: &Auth) -> Result<()> {
    match auth {
        Auth::Login => auth_login()?,
        Auth::Status => auth_status()?,
        Auth::Logout => clear_auth()?,
    }

    Ok(())
}

fn auth_login() -> Result<()> {
    Ok(())
}

fn auth_status() -> Result<()> {
    let token = get_token()?;

    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Status {
        requests_made: i32,
        expiration: u64,
    }

    let resp = minreq::get("https://cli.gai.fyi/status")
        .with_header("Authorization", format!("Bearer {}", token))
        .send()?;

    let val: Status = serde_json::from_str(resp.as_str()?)?;

    if let Some(date) = chrono::DateTime::from_timestamp(
        val.expiration
            .try_into()?,
        0,
    ) {
        println!("Requests made: {}/10", val.requests_made);
        println!("Resets at {}", date);
    } else {
        println!("Failed to convert expiration to datetime");
    }
    Ok(())
}

fn clear_auth() -> Result<()> {
    let token_path = token_path()?;

    if token_path.exists() {
        fs::remove_file(token_path)?;
    }
    println!("No longer aunthenticated");
    Ok(())
}

pub fn get_token() -> Result<String> {
    let token_path = token_path()?;
    let token = match fs::read_to_string(token_path) {
        Ok(t) => t.trim().to_string(),
        Err(e) => {
            if matches!(e.kind(), ErrorKind::NotFound) {
                return Err(anyhow::anyhow!(
                    "Token not found, have you tried logging in with: gai auth login?"
                ));
            } else {
                return Err(e.into());
            }
        }
    };

    Ok(token)
}

fn _store_token(token: &str) -> Result<()> {
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

    Ok(cfg_dir
        .config_dir()
        .join(".token"))
}
