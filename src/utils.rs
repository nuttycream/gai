use std::collections::HashMap;

use crate::{
    config::{Config, RuleConfig},
    consts::{COMMIT_CONVENTION, DEFAULT_SYS_PROMPT},
};

pub fn build_diffs_string(diffs: HashMap<String, String>) -> String {
    let mut diffs_str = String::new();

    for (file, diff) in diffs {
        let file_diff =
            format!("FileName:{}\nContent:{}\n\n", file, diff);
        diffs_str.push_str(&file_diff);
    }

    diffs_str
}

/// builds the semi-complete prompt
pub fn build_prompt(cfg: &Config) -> String {
    let mut prompt = String::new();

    let rules = build_rules(&cfg.ai.rules);

    if let Some(sys_prompt) = &cfg.ai.system_prompt {
        prompt.push_str(sys_prompt);
    } else {
        prompt.push_str(DEFAULT_SYS_PROMPT);
    };

    prompt.push('\n');

    prompt.push_str(&rules);
    prompt.push('\n');

    if cfg.ai.include_convention {
        if let Some(commit_conv) = &cfg.ai.commit_convention {
            prompt.push_str(commit_conv);
        } else {
            prompt.push_str(COMMIT_CONVENTION);
        }

        prompt.push('\n');
    }

    if cfg.gai.stage_hunks {
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

    // get repo tree is builtin to gai.
    // todo make it independent
    // + this build_prompt and build_rules
    // should be in ai/
    // since it'll only be used there
    /* if cfg.ai.include_file_tree {
        prompt.push_str(get_repo_tree);
    } */

    prompt.push('\n');

    prompt
}

fn build_rules(cfg: &RuleConfig) -> String {
    let mut rules = String::new();

    if cfg.group_related_files {
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

    if cfg.no_file_splitting {
        rules
            .push_str("- Each file should appear in ONLY ONE commit");
    }

    if cfg.separate_by_purpose {
        rules.push_str("- Create SEPARATE commits when changes serve DIFFERENT purposes");
    }

    rules.push_str("- For CommitMessages:");
    rules.push_str(
        "  * prefix: The appropriate type from the PrefixType enum",
    );

    let header = format!(
        "  * header: Keep under {} characters total (including type and scope)",
        cfg.max_header_length
    );
    rules.push_str(&header);

    let body = format!(
        "  * body: Wrap lines at {} characters. Provide detailed context.",
        cfg.max_body_length
    );
    rules.push_str(&body);

    if cfg.allow_empty_scope {
        if cfg.exclude_extension_in_scope {
            rules.push_str("  * scope: The component name or \"\", DO NOT include the file extension");
        } else {
            rules.push_str("  * scope: The component name or \"\"");
        }
    } else if cfg.exclude_extension_in_scope {
        rules.push_str("  * scope: The component name, DO NOT include the file extension");
    } else {
        rules.push_str("  * scope: The component name");
    }

    rules.push_str(
        "  * breaking: true if breaking change, false otherwise",
    );

    if cfg.verbose_descriptions {
        rules.push_str("  * message: ONLY the description, do NOT include prefix or scope in the message text. \
                Make sure your descriptions are ACCURATE and VERBOSE that closely align with the changes.");
    } else {
        rules.push_str("  * message: ONLY the description, do NOT include prefix or scope in the message text.");
    }

    rules.push('\n');

    rules
}
