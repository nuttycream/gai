use console::style;
use serde_json::Value;

use crate::{
    args::{GlobalArgs, RebaseArgs},
    git::{
        DiffStrategy, GitRepo, StagingStrategy, StatusStrategy,
        branch::{find_divergence_branch, validate_branch_exists},
        commit::{GitCommit, apply_commits},
        diffs::{FileDiff, get_diffs},
        log::get_logs,
    },
    print::{
        commits::print_response_commits, loading,
        query::print_retry_prompt,
    },
    providers::{extract_from_provider, provider::ProviderKind},
    requests::rebase::create_rebase_request,
    responses::{
        commit::process_commit, rebase::parse_from_rebase_schema,
    },
    schema::{SchemaSettings, rebase::create_rebase_schema},
    state::State,
};

pub fn run(
    args: &RebaseArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    // get from branch name
    // get onto branch , defaults to head
    // get list of commits from the branch
    // maybe with get_logs()?
    //
    // collect diffs from commits
    //
    // should we send as logs?
    // or as a giant diff?
    //
    // if a giant diff, then we can
    // reuse commitschema
    // to generate a list of commits
    // to apply to onto
    //
    // if send as logs, how is that going
    // to be handled, should we create
    // a schema, and just edit the commit messages
    // from logs?

    // create the request
    // send the request + schema
    // parse response
    // prompt the user
    // to rebase on top as commits
    // or merge commits?

    let mut state = State::new(
        global
            .config
            .as_deref(),
    )?;

    if !validate_branch_exists(&state.git.repo, &args.branch)? {
        println!(
            "Branch {}, {}",
            style(&args.branch).bold(),
            style("does not exist or is an invalid branch name")
                .red()
        );

        return Ok(());
    }

    if let Some(onto) = &args.onto
        && !validate_branch_exists(&state.git.repo, onto)?
    {
        println!(
            "Branch {}, {}",
            style(&onto).bold(),
            style("does not exist or is an invalid branch name")
                .red()
        );

        return Ok(());
    }

    let diverging_commit =
        find_divergence_branch(&state.git.repo, &args.branch)?
            .to_string();

    // collected logs from diverging branch
    let logs = get_logs(
        &state.git,
        true,
        false,
        // count shouldn't
        // matter considering, we
        // pick from_hash
        0,
        false,
        Some(&diverging_commit),
        None,
        None,
    )?;

    //println!("{:#?}", logs);

    let mut log_strs = Vec::new();

    for (idx, log) in logs
        .git_logs
        .iter()
        .enumerate()
    {
        let files: String = log.files.join(",");

        let item = format!(
            "CommitID:[{}]\nCommitMessage:{}\nFiles:{}",
            idx, log.raw, files
        );

        log_strs.push(item);
    }

    let status_strategy = if state
        .settings
        .commit
        .only_staged
    {
        StatusStrategy::Stage
    } else {
        StatusStrategy::default()
    };

    let diff_strategy = DiffStrategy {
        status_strategy,
        ..Default::default()
    };

    let schema_settings = if matches!(
        state
            .settings
            .provider,
        ProviderKind::OpenAI
    ) {
        SchemaSettings::default()
            .additional_properties(false)
            .allow_min_max_ints(true)
    } else {
        SchemaSettings::default().allow_min_max_ints(true)
    };

    // FIXME: this only grabs diffs of current repo state
    // needs to get diffs from point of divergence
    // aka diverging_commit
    state.diffs = get_diffs(&state.git, &diff_strategy)?;

    let req = create_rebase_request(&state.settings, &log_strs);

    let schema = create_rebase_schema(
        schema_settings,
        &state.settings,
        &state
            .diffs
            .as_files(),
        &state
            .diffs
            .as_hunks(),
    )?;

    loop {
        let loading = loading::Loading::new(
            "Generating Commits",
            global.compact,
        )?;

        loading.start();

        let response: Value = match extract_from_provider(
            &state
                .settings
                .provider,
            req.to_owned(),
            schema.to_owned(),
        ) {
            Ok(r) => r,
            Err(e) => {
                let msg = format!(
                    "Gai received an error from the provider:\n{:#}\nRetry?",
                    e
                );

                loading.stop();

                if print_retry_prompt(Some(&msg))? {
                    continue;
                } else {
                    break;
                }
            }
        };

        let raw_commits = parse_from_rebase_schema(
            response,
            &state
                .settings
                .staging_type,
        )?;

        println!(
            "Done! Received {} Commit{}",
            raw_commits.len(),
            if raw_commits.len() == 1 { "" } else { "s" }
        );

        let selected = print_response_commits(
            &raw_commits,
            global.compact,
            matches!(
                state
                    .settings
                    .staging_type,
                StagingStrategy::Hunks
            ),
            false,
        )?;

        let git_commits: Vec<GitCommit> = raw_commits
            .into_iter()
            .map(|c| process_commit(c, &state.settings))
            .collect();

        let selected = match selected {
            Some(s) => s,
            None => {
                if apply(
                    &state.git,
                    &git_commits,
                    &mut state.diffs.files,
                    &state
                        .settings
                        .staging_type,
                )? {
                    continue;
                }
                0
            }
        };

        if selected == 0 {
            if apply(
                &state.git,
                &git_commits,
                &mut state.diffs.files,
                &state
                    .settings
                    .staging_type,
            )? {
                continue;
            }
        } else if selected == 1 {
            println!("Regenerating");
            continue;
        } else if selected == 2 {
            println!("Exiting");
        }

        loading.stop();
    }

    Ok(())
}

fn apply(
    repo: &GitRepo,
    git_commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_stragey: &StagingStrategy,
) -> anyhow::Result<bool> {
    println!("Applying Commits...");
    match apply_commits(
        &repo.repo,
        git_commits,
        og_file_diffs,
        staging_stragey,
    ) {
        Ok(_) => Ok(false),
        Err(e) => {
            let msg = format!("Failed to Apply Commits: {}", e);

            if print_retry_prompt(Some(&msg))? {
                println!("Regenerating...");
                Ok(true)
            } else {
                println!("Exiting");
                Ok(false)
            }
        }
    }
}
