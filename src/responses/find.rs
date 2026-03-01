use crate::schema::find::FindCommitSchema;

/// extract FindCommitschema from
/// response
pub fn parse_to_find_schema(
    value: serde_json::Value
) -> anyhow::Result<FindCommitSchema> {
    let val = serde_json::from_value(value)?;

    Ok(val)
}
