use serde::Deserialize;
use serde_json::Value;
use strum::VariantNames;

use crate::schema::{SchemaBuilder, SchemaSettings};

/// wrapper struct to house
#[derive(Debug, Deserialize)]
pub struct RebaseResponse {}

/// raw find schema struct, used when we
/// deserialize the response Value object
#[derive(Clone, Debug, Deserialize)]
pub struct RebaseSchema {}

pub fn create_rebase_schema(
    schema_settings: SchemaSettings
) -> anyhow::Result<Value> {
    let builder = SchemaBuilder::new().settings(schema_settings);

    let schema = builder.build();

    Ok(schema)
}
