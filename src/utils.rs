use std::collections::HashMap;

use crate::{
    config::{Config, RuleConfig},
    consts::{
        COMMIT_CONVENTION, DEFAULT_SYS_PROMPT, RULE_BODY_BASE,
        RULE_BREAKING, RULE_COMMIT_MESSAGE_HEADER, RULE_GROUP_FILES,
        RULE_HEADER_BASE, RULE_MESSAGE_CONCISE, RULE_MESSAGE_VERBOSE,
        RULE_NO_FILE_SPLITTING, RULE_PREFIX,
        RULE_SCOPE_ALLOW_EMPTY_NO_EXTENSION,
        RULE_SCOPE_ALLOW_EMPTY_WITH_EXTENSION,
        RULE_SCOPE_REQUIRED_NO_EXTENSION,
        RULE_SCOPE_REQUIRED_WITH_EXTENSION, RULE_SEPARATE_BY_PURPOSE,
    },
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
        rules.push_str(RULE_GROUP_FILES);
    }

    if cfg.no_file_splitting {
        rules.push_str(RULE_NO_FILE_SPLITTING);
    }

    if cfg.separate_by_purpose {
        rules.push_str(RULE_SEPARATE_BY_PURPOSE);
    }

    rules.push_str(RULE_COMMIT_MESSAGE_HEADER);
    rules.push_str(RULE_PREFIX);

    let scope_rule =
        match (cfg.allow_empty_scope, cfg.exclude_extension_in_scope)
        {
            (true, true) => RULE_SCOPE_ALLOW_EMPTY_NO_EXTENSION,
            (true, false) => RULE_SCOPE_ALLOW_EMPTY_WITH_EXTENSION,
            (false, true) => RULE_SCOPE_REQUIRED_NO_EXTENSION,
            (false, false) => RULE_SCOPE_REQUIRED_WITH_EXTENSION,
        };
    rules.push_str(scope_rule);

    rules.push_str(RULE_BREAKING);

    rules.push_str(RULE_HEADER_BASE);
    rules.push_str(&format!(
        "    - CRITICAL: Maximum length is {} characters\n",
        cfg.max_header_length
    ));

    rules.push_str(RULE_BODY_BASE);
    rules.push_str(&format!(
        "    - Wrap lines at {} characters\n",
        cfg.max_body_length
    ));

    if cfg.verbose_descriptions {
        rules.push_str(RULE_MESSAGE_VERBOSE);
    } else {
        rules.push_str(RULE_MESSAGE_CONCISE);
    }

    rules.push('\n');
    rules
}
