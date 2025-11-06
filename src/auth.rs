use std::{fs, path::PathBuf};

use anyhow::Result;

pub fn store_token(token: &str) -> Result<()> {
    if token.is_empty() {
        return Err(anyhow::anyhow!("token cannot be empty"));
    }

    let token_path = token_path()?;

    fs::write(&token_path, token)?;

    Ok(())
}

pub fn get_token() -> Result<String> {
    let token_path = token_path()?;

    Ok(fs::read_to_string(token_path)?.trim().to_string())
}

pub fn delete_token() -> Result<()> {
    let token_path = token_path()?;

    if token_path.exists() {
        fs::remove_file(token_path)?;
    }

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
