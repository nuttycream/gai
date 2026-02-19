pub mod branch;
pub mod interactive;
pub mod last;
pub mod plan;
pub mod range;

use git2::Oid;
use serde_json::Value;

use crate::{
    args::{GlobalArgs, RebaseArgs},
    cmd::rebase::plan::gen_plan,
    git::{
        GitRepo, StagingStrategy,
        commit::{GitCommit, apply_commits},
        diffs::{FileDiff, get_diffs_from_commits},
        log::get_logs,
        rebase::cherry_pick_commits,
        reset::{reset_repo_hard, reset_repo_mixed},
        status::is_workdir_clean,
        utils::get_head_repo,
    },
    print::{
        commits::print_response_commits, loading,
        print_choice_prompt, query::print_retry_prompt,
    },
    providers::{extract_from_provider, provider::ProviderKind},
    requests::rebase::create_rebase_request,
    responses::{
        commit::process_commit, rebase::parse_from_rebase_schema,
    },
    schema::{SchemaSettings, rebase::create_rebase_schema},
    state::State,
};

use super::rebase::{
    branch::rebase_branch, last::rebase_last, range::rebase_range,
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
        global,
    )?;

    if !is_workdir_clean(&state.git.repo)? {
        return Err(anyhow::anyhow!(
            "Workdir is NOT clean, please save your changes"
        ));
    }

    //println!("{:#?}", state.settings);

    // save the original point, in case
    // we need to revert back hard
    // used for reset_repo_hard
    let original_head = get_head_repo(&state.git.repo)?.to_string();

    let mut to_oid: Option<String> = None;
    let mut trailing_commits: Option<Vec<String>> = None;

    let diverge_from = if let Some(ref div_branch_arg) = args.branch {
        if let Some(oid) = rebase_branch(
            &state.git,
            Some(div_branch_arg),
            false,
            global.compact,
        )? {
            oid
        } else {
            return Ok(());
        }
    } else if let Some(last_n) = args.last {
        if let Some(oid) =
            rebase_last(&state.git, false, Some(last_n))?
        {
            oid
        } else {
            return Ok(());
        }
    } else if let Some(ref from_hash) = args.from {
        match rebase_range(
            &state.git,
            Some(from_hash),
            args.to.as_deref(),
            false,
        )? {
            Some(rebase_range) => {
                to_oid = rebase_range.to;
                trailing_commits = rebase_range.trailing;

                rebase_range.from
            }
            None => return Ok(()),
        }
    } else {
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

        match selected_flow {
            0 => {
                // handle commits since divergence
                // user can pick a branch, then run the logic
                // to find where it diverged from head
                // colllect all commits from that point to head
                match rebase_branch(
                    &state.git,
                    None,
                    true,
                    global.compact,
                )? {
                    Some(oid) => oid,
                    None => return Ok(()),
                }
            }
            1 => {
                // handle specify last N fo commits
                // pretty straightfoward, prompt for
                // count, specify max,
                match rebase_last(&state.git, true, None)? {
                    Some(oid) => oid,
                    None => return Ok(()),
                }
            }
            2 => {
                // handle commit range
                // use something akin to print_query_logs()
                // first bring up the query logs
                // to fuzzy find a commit from_hash
                // then use it again for to_hash
                match rebase_range(&state.git, None, None, true)? {
                    Some(rebase_range) => {
                        to_oid = rebase_range.to;
                        trailing_commits = rebase_range.trailing;

                        rebase_range.from
                    }
                    None => return Ok(()),
                }
            }
            _ => unreachable!(),
        }
    };

    // collect logs
    let logs = get_logs(
        &state.git,
        // FIXME: settings should override this
        true,
        // not going to include diffs, as
        // they should be unified diff
        false,
        // count limits specified
        // from hash ranges
        0,
        false,
        Some(&diverge_from.to_string()),
        to_oid.as_deref(),
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

    if let Some(ref to) = to_oid {
        // reset hard to the TO commit
        reset_repo_hard(&state.git.repo, to)?;
    }

    // do a mixed reset to the FROM commit
    reset_repo_mixed(&state.git.repo, &diverge_from.to_string())?;

    // collect diffs from the diverging_commit
    state.diffs = get_diffs_from_commits(
        &state.git.repo,
        &state.git.workdir,
        diverge_from,
        None,
    )?;

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

    // plan requires different schemas, and looping workflow
    if args.plan {
        return gen_plan(
            &state.settings,
            &state.diffs,
            &log_strs,
            &schema_settings,
        );
    }

    let request = create_rebase_request(
        &state.settings,
        &log_strs,
        &state
            .diffs
            .to_string(),
    );

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

    //println!("{:#}", schema);

    // an huge chunk of diffs are generated here
    // if the branch is ahead by LOTS of changes
    // in this case, setting a specific limit in terms
    // of the specific commit to go back from should be in place
    //println!("{}", state.diffs);

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

        loading.stop();

        println!(
            "Done! Received {} Commit{}",
            raw_commits.len(),
            if raw_commits.len() == 1 { "" } else { "s" }
        );

        let selected = match print_response_commits(
            &raw_commits,
            global.compact,
            matches!(
                state
                    .settings
                    .staging_type,
                StagingStrategy::Hunks
            ),
            false,
        )? {
            Some(s) => s,
            None => {
                println!("Exiting...");
                return Ok(());
            }
        };

        let git_commits: Vec<GitCommit> = raw_commits
            .into_iter()
            .map(|c| process_commit(c, &state.settings))
            .collect();

        if selected == 0 {
            match apply(
                &state.git,
                &git_commits,
                &mut state.diffs.files,
                &state
                    .settings
                    .staging_type,
                to_oid.as_deref(),
                trailing_commits.as_deref(),
            ) {
                // done
                Ok(false) => break,
                Ok(true) => {
                    // wants to retry
                    reset_repo_hard(&state.git.repo, &original_head)?;

                    // redo the scoped reset sequence
                    if let Some(ref to) = to_oid {
                        reset_repo_hard(&state.git.repo, to)?;
                    }
                    reset_repo_mixed(
                        &state.git.repo,
                        &diverge_from.to_string(),
                    )?;

                    continue;
                }
                Err(e) => {
                    // ideally restore on errors
                    reset_repo_hard(&state.git.repo, &original_head)?;
                    return Err(e);
                }
            }
        } else if selected == 1 {
            println!("Regenerating");
            continue;
        } else if selected == 2 {
            println!("Exiting");
            break;
        }
    }

    Ok(())
}

fn apply(
    repo: &GitRepo,
    git_commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_stragey: &StagingStrategy,
    to_oid: Option<&str>,
    trailing: Option<&[String]>,
) -> anyhow::Result<bool> {
    println!("Applying Commits...");
    match apply_commits(
        &repo.repo,
        git_commits,
        og_file_diffs,
        staging_stragey,
    ) {
        Ok(_) => {
            // after applying check if we have to_oid and trailing
            // then re-apply them
            if let Some(to) = to_oid {
                // get the tree from when it was
                // still correct
                let original_tree = repo
                    .repo
                    .find_commit(Oid::from_str(to)?)?
                    .tree()?
                    .id();

                let new_tree = repo
                    .repo
                    .head()?
                    .peel_to_tree()?
                    .id();

                //temp
                if original_tree != new_tree {
                    return Err(anyhow::anyhow!(
                        "bad trees, failed to apply correct changes"
                    ));
                }

                // reapply commits
                if let Some(trails) = trailing {
                    cherry_pick_commits(&repo.repo, trails)?;
                }
            }

            Ok(false)
        }
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
