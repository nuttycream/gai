use serde::{Deserialize, Serialize};

use super::provider::AI;

/// this is rules/constraints to send the ai
/// along with the prompt
#[derive(Debug, Serialize, Deserialize)]
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

    // todo add hard validation
    pub max_header_length: u16,
    pub max_body_length: u16,
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

            max_header_length: 50,
            max_body_length: 72,
        }
    }
}

impl AI {
    pub fn build_rules(&self) -> String {
        let mut rules = String::new();

        if self.rules.group_related_files {
            rules.push_str("- GROUP related files into LOGICAL commits based on the type of change");
            rules.push_str("- Examples of files that should be grouped together:");
            rules.push_str(
                "  * Multiple files implementing the same feature",
            );
            rules.push_str("  * Files modified for the same bug fix");
            rules.push_str(
                "  * Related configuration and code changes",
            );
            rules.push_str("  * Test files with the code they test");
        }

        if self.rules.no_file_splitting {
            rules.push_str(
                "- Each file should appear in ONLY ONE commit",
            );
        }

        if self.rules.separate_by_purpose {
            rules.push_str("- Create SEPARATE commits when changes serve DIFFERENT purposes");
        }

        rules.push_str("- For CommitMessages:");
        rules.push_str("  * prefix: The appropriate type from the PrefixType enum");

        let header = format!(
            "  * header: Keep under {} characters total (including type and scope)",
            self.rules.max_header_length
        );
        rules.push_str(&header);

        let body = format!(
            "  * body: Wrap lines at {} characters. Provide detailed context.",
            self.rules.max_body_length
        );
        rules.push_str(&body);

        if self.rules.allow_empty_scope {
            if self.rules.exclude_extension_in_scope {
                rules.push_str("  * scope: The component name or \"\", DO NOT include the file extension");
            } else {
                rules.push_str(
                    "  * scope: The component name or \"\"",
                );
            }
        } else if self.rules.exclude_extension_in_scope {
            rules.push_str("  * scope: The component name, DO NOT include the file extension");
        } else {
            rules.push_str("  * scope: The component name");
        }

        rules.push_str(
            "  * breaking: true if breaking change, false otherwise",
        );

        if self.rules.verbose_descriptions {
            rules.push_str("  * message: ONLY the description, do NOT include prefix or scope in the message text. \
                Make sure your descriptions are ACCURATE and VERBOSE that closely align with the changes.");
        } else {
            rules.push_str("  * message: ONLY the description, do NOT include prefix or scope in the message text.");
        }

        rules.push_str("\n");

        rules
    }
}
