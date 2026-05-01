pub mod branch;
pub mod last;
pub mod plan;
pub mod range;

use git2::Oid;
use serde_json::Value;

use crate::{
    args::{GlobalArgs, RebaseArgs, RebaseScope},
    cmd::{
        commit::{RESPONSE_OPTS, ResponseActions},
        rebase::plan::gen_plan,
    },
    git::{
        GitRepo, StagingStrategy,
        checkout::force_checkout_head,
        commit::{GitCommit, apply_commits},
        diffs::{FileDiff, get_diffs_from_commits},
        log::{Logs, get_logs},
        rebase::{
            cherry_pick_commits, cherry_pick_reword,
            cherry_pick_single, squash_to_head,
        },
        reset::{reset_repo_hard, reset_repo_mixed},
        status::is_workdir_clean,
        utils::get_head_repo,
    },
    print::{
        self, commits::response_commits, menu::Menu,
        spinner::SpinnerBuilder,
    },
    providers::{extract_from_provider, provider::ProviderKind},
    requests::rebase::create_rebase_request,
    responses::{
        commit::process_commit, rebase::parse_from_rebase_schema,
    },
    schema::{
        SchemaSettings,
        rebase::create_rebase_schema,
        rebase_plan::{PlanOperationKind, PlanOperationSchema},
    },
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

    print::status::provider_info(
        &state
            .settings
            .provider,
        &state
            .settings
            .providers,
    )?;

    let handle = SpinnerBuilder::new()
        .text("Generating request")
        .start();

    // save the original point, in case
    // we need to revert back hard
    // used for reset_repo_hard
    let original_head = get_head_repo(&state.git.repo)?.to_string();

    let mut to_oid: Option<String> = None;
    let mut trailing_commits: Option<Vec<String>> = None;

    let diverge_from = match &args.scope {
        RebaseScope::Branch { name } => {
            match rebase_branch(&state.git, Some(name), false)? {
                Some(oid) => oid,
                None => return Ok(()),
            }
        }
        RebaseScope::Last { count } => {
            match rebase_last(&state.git, false, Some(*count))? {
                Some(oid) => oid,
                None => return Ok(()),
            }
        }
        RebaseScope::Range { from, to } => {
            match rebase_range(
                &state.git,
                Some(from),
                to.as_deref(),
                false,
            )? {
                Some(r) => {
                    to_oid = r.to;
                    trailing_commits = r.trailing;
                    r.from
                }
                None => return Ok(()),
            }
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
        // should be oldest first
        true,
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

    let to = to_oid
        .as_deref()
        .map(Oid::from_str)
        .transpose()?
        .unwrap_or(get_head_repo(&state.git.repo)?);

    // collect diffs from the diverging_commit
    state.diffs = get_diffs_from_commits(
        &state.git.repo,
        &state.git.workdir,
        diverge_from,
        Some(to),
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
        match gen_plan(
            &state.settings,
            &state.diffs,
            &log_strs,
            &schema_settings,
        )? {
            Some(ops) => {
                // reset to the from commit
                // since, compared to the
                // commit generation apply()
                // im not using the diffs/changes
                // but instead the existing commits
                reset_repo_hard(
                    &state.git.repo,
                    &diverge_from.to_string(),
                )?;

                match apply_plan(
                    &state.git,
                    &ops,
                    &logs,
                    trailing_commits.as_deref(),
                ) {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        println!(
                            "couldnt apply plan: {}\nresetting",
                            e
                        );

                        reset_repo_hard(
                            &state.git.repo,
                            &original_head,
                        )?;
                    }
                }

                return Ok(());
            }
            None => {
                reset_repo_hard(&state.git.repo, &original_head)?;
                return Ok(());
            }
        }
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

    handle.done();

    loop {
        let handle = SpinnerBuilder::new()
            .text("Generating commits")
            .start();

        let response: Value = match extract_from_provider(
            &state
                .settings
                .provider,
            request.to_owned(),
            schema.to_owned(),
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error from the provider:\n{:#}", e);

                break;
            }
        };

        let mut raw_commits = parse_from_rebase_schema(
            response,
            &state
                .settings
                .staging_type,
        )?;

        handle.done();

        response_commits(
            &raw_commits,
            matches!(
                state
                    .settings
                    .staging_type,
                StagingStrategy::Hunks
            ),
        )?;

        let mut regenerate = false;

        loop {
            let selected =
                Menu::new("What do you want to do?", &RESPONSE_OPTS)
                    .render()?;

            match selected {
                ResponseActions::Apply => {
                    let git_commits: Vec<GitCommit> = raw_commits
                        .iter()
                        .cloned()
                        .map(|c| process_commit(c, &state.settings))
                        .collect();

                    if let Some(ref to) = to_oid {
                        // reset hard to the TO commit
                        reset_repo_hard(&state.git.repo, to)?;
                    }

                    // do a mixed reset to the FROM commit
                    reset_repo_mixed(
                        &state.git.repo,
                        &diverge_from.to_string(),
                    )?;

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
                            reset_repo_hard(
                                &state.git.repo,
                                &original_head,
                            )?;

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
                            reset_repo_hard(
                                &state.git.repo,
                                &original_head,
                            )?;
                            return Err(e);
                        }
                    }
                }
                ResponseActions::Regen => {
                    regenerate = true;
                    break;
                }
                ResponseActions::Edit => {
                    raw_commits = crate::cmd::commit::edit_commits(
                        &raw_commits,
                    )?;

                    if raw_commits.is_empty() {
                        break;
                    }

                    print::commits::response_commits(
                        &raw_commits,
                        matches!(
                            state
                                .settings
                                .staging_type,
                            StagingStrategy::Hunks
                        ),
                    )?;

                    continue;
                }
                ResponseActions::Quit => {
                    break;
                }
            }
        }

        if regenerate {
            continue;
        }

        break;
    }

    Ok(())
}

fn apply_plan(
    git: &GitRepo,
    ops: &[PlanOperationSchema],
    logs: &Logs,
    trailing: Option<&[String]>,
) -> anyhow::Result<()> {
    // reordering commits by index
    // in case the LLM decides to do so
    // im keeping this off limits for now
    // since, idk how im gonna handle the conflicts
    // ideally, with drop as well
    // or should it be an option along with Drop?
    let mut ops = ops.to_vec();
    ops.sort_by_key(|op| op.commit_index);

    for op in ops {
        let commit =
            &logs.git_logs[op.commit_index as usize].commit_hash;

        match op.operation {
            PlanOperationKind::Pick => {
                cherry_pick_single(&git.repo, commit)?;
            }
            PlanOperationKind::Squash => {
                let message = if let Some(ref msg) = op.new_message {
                    msg
                } else {
                    return Err(anyhow::anyhow!(
                        "no message in schema for a squash, not good, bailing"
                    ));
                };

                squash_to_head(&git.repo, commit, message)?;
            }
            PlanOperationKind::Reword => {
                let message = if let Some(ref msg) = op.new_message {
                    msg
                } else {
                    return Err(anyhow::anyhow!(
                        "no message in schema for a reword, not good, bailing"
                    ));
                };

                cherry_pick_reword(&git.repo, commit, message)?;
            }
            PlanOperationKind::Drop => {
                // do nothing
            }
        }
    }

    if let Some(trails) = trailing {
        cherry_pick_commits(&git.repo, trails)?;
    }

    // sync it
    force_checkout_head(&git.repo)?;

    Ok(())
}

fn apply(
    git: &GitRepo,
    git_commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_stragey: &StagingStrategy,
    to_oid: Option<&str>,
    trailing: Option<&[String]>,
) -> anyhow::Result<bool> {
    println!("Applying Commits...");
    match apply_commits(
        &git.repo,
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
                let original_tree = git
                    .repo
                    .find_commit(Oid::from_str(to)?)?
                    .tree()?
                    .id();

                let new_tree = git
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
                    cherry_pick_commits(&git.repo, trails)?;
                }
            }

            Ok(false)
        }
        Err(e) => {
            eprintln!("failed to apply commits:\n{e}");

            Ok(false)
        }
    }
}
