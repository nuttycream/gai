use crate::config::RuleConfig;

impl RuleConfig {
    pub fn build_rules(&self) -> String {
        let mut rules = String::new();

        if self.group_related_files {
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

        if self.no_file_splitting {
            rules.push_str(
                "- Each file should appear in ONLY ONE commit",
            );
        }

        if self.separate_by_purpose {
            rules.push_str("- Create SEPARATE commits when changes serve DIFFERENT purposes");
        }

        rules.push_str("- For CommitMessages:");
        rules.push_str("  * prefix: The appropriate type from the PrefixType enum");

        let header = format!(
            "  * header: Keep under {} characters total (including type and scope)",
            self.max_header_length
        );
        rules.push_str(&header);

        let body = format!(
            "  * body: Wrap lines at {} characters. Provide detailed context.",
            self.max_body_length
        );
        rules.push_str(&body);

        if self.allow_empty_scope {
            if self.exclude_extension_in_scope {
                rules.push_str("  * scope: The component name or \"\", DO NOT include the file extension");
            } else {
                rules.push_str(
                    "  * scope: The component name or \"\"",
                );
            }
        } else if self.exclude_extension_in_scope {
            rules.push_str("  * scope: The component name, DO NOT include the file extension");
        } else {
            rules.push_str("  * scope: The component name");
        }

        rules.push_str(
            "  * breaking: true if breaking change, false otherwise",
        );

        if self.verbose_descriptions {
            rules.push_str("  * message: ONLY the description, do NOT include prefix or scope in the message text. \
                Make sure your descriptions are ACCURATE and VERBOSE that closely align with the changes.");
        } else {
            rules.push_str("  * message: ONLY the description, do NOT include prefix or scope in the message text.");
        }

        rules.push('\n');

        rules
    }
}
