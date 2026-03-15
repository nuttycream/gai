use serde::Deserialize;
use serde_json::Value;
use strum::VariantNames;

use crate::{
    schema::{
        SchemaBuilder, SchemaSettings,
        commit::{CommitSchema, PrefixType},
    },
    settings::Settings,
};

/// wrapper struct for the reword response
/// schema to deserialize from
#[derive(Debug, Deserialize)]
pub struct RewordResponse {
    #[serde(default)]
    pub commit_messages: Vec<CommitSchema>,
}

/// creates a schema for new commit messages
/// following the proper format
/// somewhat mimics create commit schema
/// with tweaks removing the staging portion
pub fn create_reword_schema(
    schema_settings: SchemaSettings,
    settings: &Settings,
) -> anyhow::Result<Value> {
    let mut builder = SchemaBuilder::new()
        .settings(schema_settings.to_owned())
        .insert_str(
            "reasoning",
            Some("reason why you decided to chose this specific commit"),
            true,
        );

    builder.add_enum(
        "prefix",
        Some("conventional commit type"),
        true,
        PrefixType::VARIANTS,
    );

    if settings
        .commit
        .include_scope
    {
        builder.add_str("scope", Some("scope of the change"), true);
    }

    if settings
        .commit
        .include_breaking
    {
        builder.add_bool(
            "breaking",
            Some("is this a breaking change?"),
            true,
        );
    }

    builder.add_str("header", Some("short commit description"), true);

    if settings
        .rules
        .allow_body
    {
        builder.add_str("body", Some("extended description"), true);
    }

    let schema = builder.build();

    Ok(schema)
}
