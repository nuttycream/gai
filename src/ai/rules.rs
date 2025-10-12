use serde::{Deserialize, Serialize};

use super::provider::AI;

/// this is rules/constraints to send the ai
/// along with the prompt
#[derive(Serialize, Deserialize)]
pub struct RuleConfig {
    /// group related files into logical commits based on the type of prefix
    pub group_related_files: bool,

    /// dont split single files, each file should be in ONE commit
    /// for hunk staging, this may be ignored imo, otherwise
    /// might have to keep this perma true
    pub no_file_splitting: bool,

    /// create SEPARATE commits when changes serve different purposes
    /// as in dont lump unrelated changes into one commit
    pub separate_by_purpose: bool,

    /// llm based verbosity
    pub verbose_descriptions: bool,

    /// file extensions in scope feat(git.rs) vs feat(git)
    pub exclude_extension_in_scope: bool,

    /// empty scope scope can be "" in the response
    pub allow_empty_scope: bool,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            group_related_files: true,
            no_file_splitting: true,
            separate_by_purpose: true,
            verbose_descriptions: true,
            exclude_extension_in_scope: true,
            allow_empty_scope: true,
        }
    }
}

impl AI {
    pub fn build_rules(&self) -> String {
        let mut rules = Vec::new();

        if self.rules.group_related_files {
            rules.push("- GROUP related files into LOGICAL commits based on the type of change");
            rules.push("- Examples of files that should be grouped together:");
            rules.push(
                "  * Multiple files implementing the same feature",
            );
            rules.push("  * Files modified for the same bug fix");
            rules.push("  * Related configuration and code changes");
            rules.push("  * Test files with the code they test");
        }

        if self.rules.no_file_splitting {
            rules
                .push("- Each file should appear in ONLY ONE commit");
        }

        if self.rules.separate_by_purpose {
            rules.push("- Create SEPARATE commits when changes serve DIFFERENT purposes");
        }

        rules.push("- For CommitMessages:");
        rules.push("  * prefix: The appropriate type from the PrefixType enum");

        if self.rules.allow_empty_scope {
            if self.rules.exclude_extension_in_scope {
                rules.push("  * scope: The component name or \"\", DO NOT include the file extension");
            } else {
                rules.push("  * scope: The component name or \"\"");
            }
        } else if self.rules.exclude_extension_in_scope {
            rules.push("  * scope: The component name, DO NOT include the file extension");
        } else {
            rules.push("  * scope: The component name");
        }

        rules.push(
            "  * breaking: true if breaking change, false otherwise",
        );

        if self.rules.verbose_descriptions {
            rules.push("  * message: ONLY the description, do NOT include prefix or scope in the message text. \
                Make sure your descriptions are ACCURATE and VERBOSE that closely align with the changes.");
        } else {
            rules.push("  * message: ONLY the description, do NOT include prefix or scope in the message text.");
        }

        rules.join("\n")
    }
}
