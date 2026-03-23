use crate::schema::commit::{CommitSchema, PrefixType};

use super::{
    renderer::Renderer,
    tree::{Tree, TreeItem},
};

pub(crate) fn schemas(
    renderer: &Renderer,
    commits: &[CommitSchema],
) -> anyhow::Result<()> {
    Ok(())
}

/// display the responsecommits
/// before converting to usable
/// git commits
/// returns an selected option
pub fn print_response_commits(
    commits: &[CommitSchema],
    compact: bool,
    as_hunks: bool,
    skip_confirmation: bool,
) -> anyhow::Result<Option<usize>> {
    Ok(None)
}
