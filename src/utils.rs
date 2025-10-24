use crate::consts::COMMIT_CONVENTION;

pub fn build_rules(
    group_related_files: bool,
    no_file_splitting: bool,
    separate_by_purpose: bool,
    verbose_descriptions: bool,
    exclude_extension_in_scope: bool,
    allow_empty_scope: bool,

    max_header_len: u16,
    max_body_len: u16,
) -> String {
    let mut rules = String::new();

    if group_related_files {
        rules.push_str("- GROUP related files into LOGICAL commits based on the type of change");
        rules.push_str(
            "- Examples of files that should be grouped together:",
        );
        rules.push_str(
            "  * Multiple files implementing the same feature",
        );
        rules.push_str("  * Files modified for the same bug fix");
        rules.push_str("  * Related configuration and code changes");
        rules.push_str("  * Test files with the code they test");
    }

    if no_file_splitting {
        rules
            .push_str("- Each file should appear in ONLY ONE commit");
    }

    if separate_by_purpose {
        rules.push_str("- Create SEPARATE commits when changes serve DIFFERENT purposes");
    }

    rules.push_str("- For CommitMessages:");
    rules.push_str(
        "  * prefix: The appropriate type from the PrefixType enum",
    );

    let header = format!(
        "  * header: Keep under {} characters total (including type and scope)",
        max_header_len
    );
    rules.push_str(&header);

    let body = format!(
        "  * body: Wrap lines at {} characters. Provide detailed context.",
        max_body_len
    );
    rules.push_str(&body);

    if allow_empty_scope {
        if exclude_extension_in_scope {
            rules.push_str("  * scope: The component name or \"\", DO NOT include the file extension");
        } else {
            rules.push_str("  * scope: The component name or \"\"");
        }
    } else if exclude_extension_in_scope {
        rules.push_str("  * scope: The component name, DO NOT include the file extension");
    } else {
        rules.push_str("  * scope: The component name");
    }

    rules.push_str(
        "  * breaking: true if breaking change, false otherwise",
    );

    if verbose_descriptions {
        rules.push_str("  * message: ONLY the description, do NOT include prefix or scope in the message text. \
                Make sure your descriptions are ACCURATE and VERBOSE that closely align with the changes.");
    } else {
        rules.push_str("  * message: ONLY the description, do NOT include prefix or scope in the message text.");
    }

    rules.push('\n');

    rules
}

pub fn build_prompt(
    use_convention: bool,
    sys_prompt: &str,
    rules: &str,
    stage_hunks: bool,
) -> String {
    let mut prompt = String::new();

    prompt.push_str(sys_prompt);
    prompt.push('\n');

    prompt.push_str(rules);
    prompt.push('\n');

    if use_convention {
        prompt.push_str(COMMIT_CONVENTION);
    }

    prompt.push('\n');

    if stage_hunks {
        prompt.push_str(
        "Fill hunk_ids with the HUNK_ID values shown in the diffs (format: \"filepath:index\").\
        Each hunk can only appear in ONE commit.\
        Ex.: [\"src/main.rs:0\", \"src/git/repo.rs:1\"]",
        );
    } else {
        prompt.push_str(
            "Fill out files with valid paths and leave hunk_headers empty",
        );
    }

    prompt.push('\n');

    prompt
}
