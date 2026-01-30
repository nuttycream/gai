use console::style;
use git2::Oid;
use serde_json::Value;

use crate::{
    args::{GlobalArgs, RebaseArgs},
    git::{
        GitRepo, StagingStrategy,
        branch::get_diverged_branches,
        commit::{GitCommit, find_parent_commit},
        diffs::{FileDiff, get_diffs_from_commits},
        log::get_logs,
        rebase::rebase_commits,
    },
    print::{
        commits::print_response_commits, loading, log::print_logs,
        print_choice_prompt, print_input_prompt,
        query::print_retry_prompt, rebase::print_branches_info,
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
        "Specify Commit Range",
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

    let diverge_from: Oid;
    let from_hash: Option<String>;

    match selected_flow {
        0 => {
            // handle commits since divergence
            // user can pick a branch, then run the logic
            // to find where it diverged from head
            // colllect all commits from that point to head
            diverge_from =
                match divergence_flow(&state.git, global.compact)? {
                    Some(oid) => oid,
                    None => return Ok(()),
                };

            from_hash = Some(diverge_from.to_string());
        }
        1 => {
            // handle specify last N fo commits
            // pretty straightfoward, prompt for
            // count, specify max,

            diverge_from = match last_n_flow(&state.git)? {
                Some(oid) => oid,
                None => return Ok(()),
            };

            from_hash = Some(diverge_from.to_string());
        }
        2 => {
            // handle commit range
            // use something akin to print_query_logs()
            // first bring up the query logs
            // to fuzzy find a commit from_hash
            // then use it again for to_hash
            let res = match specify_range_flow(&state.git)? {
                Some(r) => r,
                None => return Ok(()),
            };

            diverge_from = res.0;
            from_hash = Some(res.1);
        }
        _ => unreachable!(),
    };

    // collect logs
    let logs = get_logs(
        &state.git,
        // FIXME: args option should override this
        true,
        // not going to include diffs, as
        // they should be unified diff
        false,
        // count limits specified
        // from hash ranges
        0,
        false,
        from_hash.as_deref(),
        None,
        None,
    )?;

    //println!("{:#?}", logs);

    // collect diffs from the diverging_commit
    state.diffs = get_diffs_from_commits(
        &state.git.repo,
        &state.git.workdir,
        diverge_from,
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
    // println!("{}", state.diffs);

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

        loading.stop();

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
                    diverge_from,
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
                diverge_from,
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
    }

    Ok(())
}

fn divergence_flow(
    repo: &GitRepo,
    compact: bool,
) -> anyhow::Result<Option<Oid>> {
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

    let commit_oid = if let Some(d) = branches[selected_branch]
        .divergence
        .to_owned()
    {
        d.merge_base
    } else {
        println!(
            "No merge_base available... exiting, this shouldn't happen"
        );
        return Ok(None);
    };

    Ok(Some(commit_oid))
}

fn last_n_flow(repo: &GitRepo) -> anyhow::Result<Option<Oid>> {
    let n: usize;

    loop {
        let input =
            match print_input_prompt("Specify a valid number", None)?
            {
                Some(i) => i,
                None => {
                    println!("Exiting...");
                    return Ok(None);
                }
            };

        match input.parse::<usize>() {
            Ok(v) => {
                if v == 0 {
                    println!("Please enter a value greater than 0");
                    continue;
                }

                n = v;
                break;
            }
            Err(_) => {
                println!("Cannot parse {} as a valid number", input);
                continue;
            }
        }
    }

    let logs =
        get_logs(repo, false, false, n, false, None, None, None)?;

    // if n exceeds log length, continue, regardless
    if n > logs.git_logs.len() {
        println!(
            "Only {} commits exist in history but you requested {}",
            style(logs.git_logs.len()).red(),
            style(n).red()
        );
    }

    // this should get the last logged commit
    // if the count exceeds, get_logs()
    // will handle that and return or "take"
    // the last commit
    let oldest_commit_hash = match logs
        .git_logs
        .last()
        .map(|l| {
            l.commit_hash
                .to_owned()
        }) {
        Some(h) => h,
        None => {
            println!("No Commits Found, Exiting...");
            return Ok(None);
        }
    };

    let oid = find_parent_commit(&repo.repo, &oldest_commit_hash)?;

    Ok(Some(oid))
}

fn specify_range_flow(
    repo: &GitRepo
) -> anyhow::Result<Option<(Oid, String)>> {
    let logs =
        get_logs(repo, false, false, 0, false, None, None, None)?;

    if logs
        .git_logs
        .is_empty()
    {
        println!("No commits found. Exiting...");
        return Ok(None);
    }

    loop {
        // logs are ordered newwest, so we use
        // older and newer terms
        // to avoid confusion with list position
        let first = match print_logs(
            &logs.git_logs,
            Some("Select the starting range"),
            Some(10),
        )? {
            Some(s) => s,
            None => {
                println!("Exiting...");
                return Ok(None);
            }
        };

        let commit = &logs.git_logs[first];

        let logs = get_logs(
            repo,
            false,
            false,
            0,
            false,
            Some(&commit.commit_hash),
            None,
            None,
        )?;

        let count = logs.git_logs.len();

        if count == 0 {
            println!(
                "No commits in selected range OR commit selected is HEAD. Resetting..."
            );
            continue;
        }

        println!(
            "{} Rebasing {} commit{} since {}:",
            style("→").green(),
            style(count).cyan(),
            if count == 1 { "" } else { "s" },
            style("HEAD").red(),
        );

        println!(
            " From: {} {}",
            style(&commit.commit_hash[..7]).dim(),
            String::from(commit.to_owned())
        );

        let diverge_from =
            find_parent_commit(&repo.repo, &commit.commit_hash)?;

        return Ok(Some((diverge_from, diverge_from.to_string())));
    }
}

fn apply(
    repo: &GitRepo,
    diverged_from: Oid,
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
