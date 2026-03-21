use crate::{schema::commit::CommitSchema, settings::Settings};

/// extract CommitSchemas from response
/// should return a List of commit schemas
/// tho the amount will differ
/// depending on the StagingStrategy
/// we can handle as is
pub fn parse_to_reword_commit_schema(
    value: serde_json::Value
) -> anyhow::Result<Vec<CommitSchema>> {
    let commits: Vec<CommitSchema> = serde_json::from_value(
        value
            .get("commits")
            .ok_or(anyhow::anyhow!(
                "No commits array in Response json"
            ))?
            .to_owned(),
    )?;

    Ok(commits)
}

// apply settings to a
// single commit using
// CommitSettings
pub fn process_reword_commit_message(
    raw_commit: CommitSchema,
    settings: &Settings,
) -> String {
    let commit_settings = &settings.commit;
    // prefix(scope)breaking: header
    //
    // body

    let prefix = if commit_settings.capitalize_prefix {
        &raw_commit
            .prefix
            .to_string()
    } else {
        &raw_commit
            .prefix
            .to_string()
            .to_lowercase()
    };

    let scope = if let Some(scope) = raw_commit.scope
        && commit_settings.include_scope
    {
        format!("({})", scope)
    } else {
        String::new()
    };

    // again, redudant
    let breaking = if raw_commit
        .breaking
        .is_some_and(|b| b)
        && commit_settings.include_breaking
    {
        commit_settings
            .breaking_symbol
            .to_string()
    } else {
        String::new()
    };

    // check is not part of commit settings
    // EXISTENCE SHOULD be handled during
    // schema creation
    if let Some(body) = raw_commit.body {
        format!(
            "{}{}{}: {}\n\n{}",
            prefix, scope, breaking, raw_commit.header, body
        )
    } else {
        format!(
            "{}{}{}: {}",
            prefix, scope, breaking, raw_commit.header
        )
    }
}
