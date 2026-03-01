use serde::Deserialize;
use serde_json::Value;

use crate::schema::{SchemaBuilder, SchemaSettings};

/// wrapper for rebaseplan responses
/// that will be deserialized into
#[derive(Debug, Deserialize)]
pub struct RebasePlanResponse {
    pub operations: Vec<PlanOperationSchema>,
}

/// rebaseplan schema components
#[derive(Clone, Debug, Deserialize)]
pub struct PlanOperationSchema {
    pub reasoning: String,
    pub commit_id: u32,
    pub operation: PlanOperationKind,
    // optional, but required for reword and squash
    pub new_message: Option<String>,
    // For now, disabling this to simplify
    // the schema, since i only want to support
    // squashing with the immediate previous commit
    // or at least only want the LLM to be able to do
    //pub squash_with: Option<u32>,
}

/// rebase operation types
#[derive(
    Clone, Debug, Deserialize, strum::Display, strum::VariantNames,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PlanOperationKind {
    /// simple pick, leave as is
    Pick,
    /// combine commits,
    /// must gen a commit message
    /// squashes_with, is other commit
    /// it will combine with
    /// for LLM sake and my own sanity
    /// we will only handle PAIRS
    /// and FIXME: add validation,
    /// so that other squashes_with
    /// does not point to the same commit
    Squash,
    /// reword a commit
    Reword,
    /// drop a commit,
    /// this is dangerous
    /// and will be left off by default
    Drop,
}

/// create a rebsase plan schema,
/// FIXME: allow_drop should be a configurable
/// option in settings.
pub fn create_rebase_plan_schema(
    schema_settings: SchemaSettings,
    max_commit_id: usize,
    allow_drop: bool,
) -> anyhow::Result<Value> {
    let max_commit_id = max_commit_id as u32;

    let builder = SchemaBuilder::new()
        .settings(schema_settings.clone())
        .insert_str(
            "reasoning",
            Some("explain why this operation was chosen for this commit"),
            true,
        )
        .insert_int(
            "commit_id",
            Some("the commit index this operation applies to"),
            true,
            Some(0),
            Some(max_commit_id),
        )
        .insert_enum(
            "operation",
            Some("the rebase operation to perform"),
            true,
            {
                if allow_drop {
                    &["pick", "squash", "reword", "drop"]
                } else {
                    &["pick", "squash", "reword"]
                }
            },
        )
        .insert_str(
            "new_message",
            Some("new commit message, THIS IS REQUIRED for reword and squash ops"),
            false,
        );

    let operation_schema = builder.build_inner();

    let schema = SchemaBuilder::new()
        .settings(schema_settings)
        .insert_object_array(
            "operations",
            Some("list of rebase operations, one per commit"),
            true,
            operation_schema,
        )
        .build();

    Ok(schema)
}
