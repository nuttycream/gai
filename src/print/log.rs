use std::fmt;

use crate::git::log::{GitLog, get_short_hash};

pub fn print_logs(
    git_logs: &[GitLog],
    prompt: Option<&str>,
    limit: Option<usize>,
) -> anyhow::Result<Option<usize>> {
    Ok(Some(0))
}
