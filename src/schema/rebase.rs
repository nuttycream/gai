use serde::Deserialize;
use serde_json::Value;

use crate::settings::Settings;

use super::{
    SchemaSettings, commit::CommitSchema,
    commit::create_commit_response_schema,
};

/// wrapper struct to house
/// rebase response
///
/// FOR now, expect a response that
/// resembles commits
/// considering, a rebase
/// "regenerates" a set of commits
/// from a diff
#[derive(Debug, Deserialize)]
pub struct RebaseResponse {
    #[serde(default)]
    pub commits: Vec<CommitSchema>,

    /// optional single commit
    /// for AllFilesOneCommit
    #[serde(default)]
    pub commit: Option<CommitSchema>,
}

/// create a rebase schema
/// for now, it'll be based on the
/// commit schema builder
/// due to the rebasing essentially
/// recreates/regenerates commits
/// from a diff
pub fn create_rebase_schema(
    schema_settings: SchemaSettings,
    settings: &Settings,
    files: &[String],
    hunk_ids: &[String],
) -> anyhow::Result<Value> {
    let schema = create_commit_response_schema(
        schema_settings,
        settings,
        files,
        hunk_ids,
    )?;

    Ok(schema)
}
