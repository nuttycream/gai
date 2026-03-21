use crate::{
    git::{
        GitRepo, StatusStrategy, log::get_logs, status::get_status,
    },
    settings::{PromptRules, Settings},
    utils::consts::*,
};

use super::Request;

pub fn create_reword_request(
    settings: &Settings,
    repo: &GitRepo,
    logs: &[String],
) -> Request {
    let prompt = build_prompt(repo, settings, logs.len());

    Request::new(&prompt).insert_contents(logs)
}

fn build_prompt(
    repo: &GitRepo,
    cfg: &Settings,
    log_count: usize,
) -> String {
    let mut prompt = String::new();

    let rules = build_rules(&cfg.rules);

    prompt.push_str(
        format!("Generate {} commit messages", log_count).as_str(),
    );

    if let Some(sys_prompt) = &cfg
        .prompt
        .system_prompt
    {
        prompt.push_str(sys_prompt);
    } else {
        prompt.push_str(DEFAULT_SYS_PROMPT);
    };

    prompt.push('\n');

    if let Some(hint) = &cfg.prompt.hint {
        prompt.push_str(
            format!("USE THIS IS A HINT FOR YOUR COMMITS: {}", hint)
                .as_str(),
        );
        prompt.push('\n');
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

    prompt.push('\n');

    if cfg
        .context
        .include_file_tree
    {
        prompt.push_str("Current File Tree: \n");
        //prompt.push_str(&git.get_repo_tree());
        prompt.push('\n');
    }

    if cfg
        .context
        .include_git_status
    {
        prompt.push_str("Current Git Status: \n");
        // todo impl separation when fmt::Display
        let staged =
            get_status(&repo.repo, &StatusStrategy::Stage).unwrap();
        let working_dir =
            get_status(&repo.repo, &StatusStrategy::WorkingDir)
                .unwrap();

        prompt.push_str(&format!("Staged\n{}", staged));
        prompt.push_str(&format!("WorkingDir\n{}", working_dir));
    }

    if cfg
        .context
        .include_log
    {
        let gai_logs = get_logs(
            repo,
            true,
            false,
            cfg.context
                .log_amount as usize,
            false,
            None,
            None,
            None,
        )
        .unwrap_or_default();

        let log_str = format!(
            "{} Recent Git Logs:\n{}\n",
            gai_logs
                .git_logs
                .len(),
            gai_logs
        );

        prompt.push_str(&log_str);
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
