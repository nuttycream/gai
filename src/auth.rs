use std::str::from_utf8;

use anyhow::Result;
use keyring::Entry;

pub fn store_token(token: &str) -> Result<()> {
    if token.is_empty() {
        return Err(anyhow::anyhow!("token cannot be empty"));
    }

    let entry = Entry::new("gai", "token")?;
    let secret = token.as_bytes();

    entry.set_secret(secret)?;
    Ok(())
}

pub fn get_token() -> Result<String> {
    let entry = Entry::new("gai", "token")?;
    let token = entry.get_secret()?;
    Ok(from_utf8(&token)?.to_string())
}
