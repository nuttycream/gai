use crate::{
    git::{StagingStrategy, commit::GitCommit},
    schema::commit::CommitSchema,
    settings::Settings,
};

/// extract CommitSchemas from response
/// should return a List of commit schemas
/// tho the amount will differ
/// depending on the StagingStrategy
/// we can handle as is
pub fn parse_to_commit_schema(
    value: serde_json::Value,
    strategy: &StagingStrategy,
) -> anyhow::Result<Vec<CommitSchema>> {
    if matches!(strategy, StagingStrategy::AllFilesOneCommit) {
        // only single commit
        let commit: CommitSchema = serde_json::from_value(
            value
                .get("commit")
                .ok_or(anyhow::anyhow!(
                    "No single commit field in Response json"
                ))?
                .to_owned(),
        )?;

        Ok(vec![commit])
    } else {
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
}

// apply settings to a
// single commit using
// CommitSettings
pub fn process_commit(
    raw_commit: CommitSchema,
    settings: &Settings,
) -> GitCommit {
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
    let message = if let Some(body) = raw_commit.body {
        format!(
            "{}{}{}: {}\n\n{}",
            prefix, scope, breaking, raw_commit.header, body
        )
    } else {
        format!(
            "{}{}{}: {}",
            prefix, scope, breaking, raw_commit.header
        )
    };

    let files = match settings.staging_type {
        StagingStrategy::OneFilePerCommit => {
            // unwrapping here, under the assumption
            // that files are correct, we can cover that later
            // during application or
            // todo add validation
            raw_commit
                .path
                .map(|p| vec![p])
                .unwrap_or_default()
        }
        StagingStrategy::AtomicCommits => raw_commit
            .paths
            .unwrap_or_default(),
        _ => {
            // do nothing for AllFilesOneCommit
            // or hunks
            // assume to cover both
            vec![]
        }
    };

    let hunk_ids = raw_commit
        .hunk_ids
        .unwrap_or_default();

    GitCommit {
        files,
        hunk_ids,
        message,
    }
}
