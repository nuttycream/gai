use crate::{git::log::GitLog, schema::find::Confidence};

pub fn print(
    commit: &GitLog,
    files: bool,
    reasoning: Option<&str>,
    confidence: Confidence,
) -> anyhow::Result<usize> {
    Ok(0)
}
