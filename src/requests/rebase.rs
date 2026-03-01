use crate::{
    git::StagingStrategy,
    settings::{PromptRules, Settings},
    utils::consts::*,
};

use super::Request;

const SYS_PROMPT: &str = "You are a Git expert tasked with rewriting commits. \
        You are supplied a diff and the past commit messages.";

pub fn create_rebase_request(
    settings: &Settings,
    git_logs: &[String],
    diffs: &str,
) -> Request {
    let prompt = build_prompt(settings);

    Request::new(&prompt)
        .insert_contents(git_logs)
        .insert_content(diffs)
}

fn build_prompt(cfg: &Settings) -> String {
    let mut prompt = String::new();

    let rules = build_rules(&cfg.rules);

    if let Some(sys_prompt) = &cfg
        .prompt
        .system_prompt
    {
        prompt.push_str(sys_prompt);
    } else {
        prompt.push_str(SYS_PROMPT);
    };

    prompt.push('\n');

    if let Some(hint) = &cfg.prompt.hint {
        prompt.push_str(
            format!("USE THIS IS A HINT FOR YOUR COMMITS: {}", hint)
                .as_str(),
        );
        prompt.push('\n');
    }

    if cfg
        .commit
        .only_staged
    {
        prompt.push_str(PROMPT_ONLY_STAGED);
    }

    prompt.push_str(&rules);
    prompt.push('\n');

    if let Some(commit_conv) = &cfg
        .prompt
        .commit_convention
    {
        prompt.push_str(commit_conv);
    }

    if cfg
        .context
        .include_convention
    {
        prompt.push_str(COMMIT_CONVENTION);
    }

    match cfg.staging_type {
        // todo impl other staging methods
        // likely during validation as well
        StagingStrategy::Hunks => prompt.push_str(PROMPT_STAGE_HUNKS),
        _ => prompt.push_str(PROMPT_STAGE_FILES),
    }

    prompt.push('\n');

    if cfg
        .context
        .include_file_tree
    {
        prompt.push_str("Current File Tree: \n");
        //prompt.push_str(&git.get_repo_tree());
        prompt.push('\n');
    }

    prompt
}

fn build_rules(cfg: &PromptRules) -> String {
    let mut rules = String::new();

    if cfg.group_related_files {
        rules.push_str(RULE_GROUP_FILES);
    }

    if cfg.separate_by_purpose {
        rules.push_str(RULE_SEPARATE_BY_PURPOSE);
    }

    rules.push_str(RULE_COMMIT_MESSAGE_HEADER);
    rules.push_str(RULE_PREFIX);

    let scope_rule =
        match (cfg.allow_empty_scope, cfg.extension_in_scope) {
            (true, true) => RULE_SCOPE_ALLOW_EMPTY_WITH_EXTENSION,
            (true, false) => RULE_SCOPE_ALLOW_EMPTY_NO_EXTENSION,
            (false, true) => RULE_SCOPE_REQUIRED_WITH_EXTENSION,
            (false, false) => RULE_SCOPE_REQUIRED_NO_EXTENSION,
        };
    rules.push_str(scope_rule);

    rules.push_str(RULE_BREAKING);

    rules.push_str(RULE_HEADER_BASE);
    rules.push_str(&format!(
        "    - CRITICAL: Maximum length is {} characters\n",
        cfg.max_header_length
    ));

    if cfg.allow_body {
        rules.push_str(RULE_BODY_BASE);
        rules.push_str(&format!(
            "    - CRITICAL: Maximum length is {} characters\n",
            cfg.max_body_length
        ));
    } else {
        rules.push_str("DO NOT CREATE A BODY, LEAVE IT BLANK");
    }

    if cfg.verbose_descriptions {
        rules.push_str(RULE_MESSAGE_VERBOSE);
    } else {
        rules.push_str(RULE_MESSAGE_CONCISE);
    }

    rules.push('\n');
    rules
}
