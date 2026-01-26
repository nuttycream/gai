use serde_json::Value;

use crate::{
    args::{GlobalArgs, RebaseArgs},
    git::{
        GitRepo, StagingStrategy,
        branch::{find_divergence_branch, get_diverged_branches},
        commit::GitCommit,
        diffs::{FileDiff, get_diffs_from_commits},
        log::get_logs,
        rebase::rebase_commits,
        repo,
    },
    print::{
        commits::print_response_commits, loading,
        print_choice_prompt, query::print_retry_prompt,
        rebase::print_branches_info,
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

    let options = [
        "Commits Since Divergence",
        "Last Number of Commits",
        "Specific Commit Range",
    ];

    let selected_flow = if let Some(s) = print_choice_prompt(
        &options,
        None,
        Some("Select a Scope for the Rebase Operation"),
    )? {
        s
    } else {
        println!("Exiting...");
        return Ok(());
    };

    match selected_flow {
        0 => match divergence_flow(&state.git, global.compact)? {
            Some(oid) => println!("{oid}"),
            None => return Ok(()),
        },
        1 => {}
        2 => {}
        _ => unreachable!(),
    }

    let branch = &args.branch;

    let diverging_commit =
        find_divergence_branch(&state.git.repo, branch)?;

    // collected logs from diverging branch
    let logs = get_logs(
        &state.git,
        true,
        false,
        // count shouldn't
        // matter considering, we
        // pick from_hash
        args.last,
        false,
        Some(&diverging_commit.to_string()),
        None,
        None,
    )?;

    //println!("{:#?}", logs);

    // collect diffs from the diverging_commit
    state.diffs = get_diffs_from_commits(
        &state.git.repo,
        &state.git.workdir,
        diverging_commit,
        None,
    )?;

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

    let request = create_rebase_request(&state.settings, &log_strs);

    //println!("{request}");

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

    //println!("{:#?}", schema);

    // an huge chunk of diffs are generated here
    // if the branch is ahead by LOTS of changes
    // in this case, setting a specific limit in terms
    // of the specific commit to go back from should be in place
    //    println!("{}", state.diffs);

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
            request.to_owned(),
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
                    diverging_commit,
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
                diverging_commit,
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

fn divergence_flow(
    repo: &GitRepo,
    compact: bool,
) -> anyhow::Result<Option<git2::Oid>> {
    let branches = get_diverged_branches(&repo.repo)?;

    let opts = print_branches_info(&branches, compact)?;

    let selected_branch = if let Some(b) =
        print_choice_prompt(&opts, None, Some("Select a Branch"))?
    {
        b
    } else {
        println!("Exiting...");
        return Ok(None);
    };

    todo!()
}

fn apply(
    repo: &GitRepo,
    diverged_from: git2::Oid,
    git_commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_stragey: &StagingStrategy,
) -> anyhow::Result<bool> {
    println!("Applying Commits...");
    match rebase_commits(
        &repo.repo,
        diverged_from,
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
