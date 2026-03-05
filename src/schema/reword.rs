use serde::Deserialize;
use serde_json::Value;

use crate::schema::{SchemaBuilder, SchemaSettings};

/// wrapper struct for the reword response
/// schema to deserialize from
#[derive(Debug, Deserialize)]
pub struct RewordResponse {
    #[serde(default)]
    pub commit_messages: Vec<CommitMsgSchema>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CommitMsgSchema {}

/// creates a schema for new commit messages
/// following the proper format
pub fn create_find_schema(
    schema_settings: SchemaSettings,
    max: u32,
) -> anyhow::Result<Value> {
    let builder = SchemaBuilder::new()
        .settings(schema_settings.to_owned())
        .insert_str(
            "reasoning",
            Some("reason why you decided to chose this specific commit"),
            true,
        )
        .insert_int(
            "commit_id",
            Some("commit index for the chosen commit"),
            true,
            Some(0),
            Some(max),
        );

    let schema = builder.build();

    Ok(schema)
}
