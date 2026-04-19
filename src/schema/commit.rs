use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{EnumIter, VariantNames};

use crate::{
    git::StagingStrategy,
    schema::{SchemaBuilder, SchemaSettings},
    settings::Settings,
};

/// wrapper struct to house Responses
#[derive(Debug, Deserialize)]
pub struct CommitResponse {
    #[serde(default)]
    pub commits: Vec<CommitSchema>,

    /// optional single commit
    /// for AllFilesOneCommit
    #[serde(default)]
    pub commit: Option<CommitSchema>,
}

/// helper util to convert from CommitResponse
/// into a CommitSchema vec
impl From<CommitResponse> for Vec<CommitSchema> {
    fn from(value: CommitResponse) -> Self {
        if let Some(c) = value.commit {
            vec![c]
        } else {
            value.commits
        }
    }
}

/// raw commit schema struct, used when we
/// deserialize the response Value object
#[derive(Clone, Debug, Deserialize)]
pub struct CommitSchema {
    /// reason why you decided to make this
    /// commit. ex. why are they grouped together?
    /// or why decide on this type of change for the
    /// diffs
    pub reasoning: String,

    /// only populated during a
    /// OneFilePerCommit strategy
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// paths to apply commit to
    /// ex. main.rs doubloon.rs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,

    // populated/used when stage_hunks
    // is enabled
    /// hunk "ids" per file
    /// using format file:index
    /// ex: src/main.rs:0
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hunk_ids: Option<Vec<String>>,

    // commit message components
    /// commit type
    pub prefix: PrefixType,

    /// scope of the change
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// is a breaking change?
    /// this lowk redudant but we'll keep it
    /// for deserialization sake
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub breaking: Option<bool>,

    /// short commit description
    /// used as a initial view
    pub header: String,

    /// extended description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

// full display
impl fmt::Display for CommitSchema {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.prefix)?;

        if let Some(ref scope) = self.scope {
            write!(f, "({})", scope)?;
        }

        if self
            .breaking
            .unwrap_or(false)
        {
            write!(f, "!")?;
        }

        write!(f, ": {}", self.header)?;

        if let Some(ref body) = self.body {
            write!(f, "\n\n{}", body)?;
        }

        Ok(())
    }
}

/// conventional commit type prefix
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    EnumIter,
    strum::Display,
    strum::VariantNames,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PrefixType {
    Feat,
    Fix,
    Refactor,
    Style,
    Test,
    Docs,
    Build,
    CI,
    Ops,
    Chore,
    // for newbranch
    // the ai may hallucinate
    // and use these
    // on non-new branch creations
    // should we even have these clankers
    // create branches?
    //Merge,
    //Revert,
}

/// creates a schema for commits
/// staging strategy
/// determines overall structure
/// which includes, whether or
/// not multiple commits are needed
pub fn create_commit_response_schema(
    schema_settings: SchemaSettings,
    settings: &Settings,
    files: &[String],
    hunk_ids: &[String],
) -> anyhow::Result<Value> {
    let mut builder = SchemaBuilder::new()
        .settings(schema_settings.to_owned())
        .insert_str(
            "reasoning",
            Some("reason why you decided to make this commit"),
            true,
        );

    match settings.staging_type {
        // only stage as hunks
        // populates as enum array for the
        // llm to multiple choose from
        StagingStrategy::Hunks => {
            builder = builder.insert_enum_array(
                "hunk_ids",
                Some("hunk IDs to stage, format: file:index (e.g. src/main.rs:0)"),
                true,
                hunk_ids,
            );
        }
        // only ONE file PER commit
        // that means that the file path
        // for this commit entry can be only
        // one, chosen from an enum
        StagingStrategy::OneFilePerCommit => {
            builder = builder.insert_enum(
                "path",
                Some("file path for this commit"),
                true,
                files,
            );
        }
        // the response can choose multiple files
        // from this enum file array
        StagingStrategy::AtomicCommits => {
            builder = builder.insert_enum_array(
                "paths",
                Some("file paths to include in this commit"),
                true,
                files,
            );
        }
        // one block of commit, this should be the
        // only modifier, that changes the ENTIRE
        // schema, to respond with ONLY ONE commit
        // not an array, in this specific match statement
        // do nothing, this will be handled later
        // down, after build the inner commit schema
        StagingStrategy::AllFilesOneCommit => {}
    }

    // builder the inner commit schema
    // this will be wrapped by a
    // new SchemaBuilder

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

    let commit_schema = builder.build_inner();

    let schema = if matches!(
        settings.staging_type,
        StagingStrategy::AllFilesOneCommit
    ) {
        SchemaBuilder::new()
            .settings(schema_settings)
            .insert_object(
                "commit",
                Some("single commit for all changes"),
                true,
                commit_schema,
            )
            .build()
    } else {
        SchemaBuilder::new()
            .settings(schema_settings)
            .insert_object_array(
                "commits",
                Some("list of commits"),
                true,
                commit_schema,
            )
            .build()
    };

    Ok(schema)
}
