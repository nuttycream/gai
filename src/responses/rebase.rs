use crate::{git::StagingStrategy, schema::commit::CommitSchema};

/// extract rebaseSChema from
/// response
pub fn parse_from_rebase_schema(
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
