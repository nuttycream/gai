use crate::{
    args::{GlobalArgs, RewordArgs, RewordScope},
    cmd::commit::{RESPONSE_OPTS, ResponseActions, edit_commits},
    git::{
        GitRepo, StagingStrategy,
        checkout::force_checkout_head,
        commit::find_parent_commit,
        log::{Logs, get_logs},
        rebase::{
            cherry_pick_commits, cherry_pick_reword, trailing_commits,
        },
        reset::reset_repo_hard,
        status::is_workdir_clean,
        utils::get_head_repo,
    },
    print::{self, menu::Menu, spinner::SpinnerBuilder},
    providers::{extract_from_provider, provider::ProviderKind},
    requests::reword::create_reword_request,
    responses::reword::{
        parse_to_reword_commit_schema, process_reword_commit_message,
    },
    schema::{SchemaSettings, reword::create_reword_schema},
    state::State,
};

pub fn run(
    args: &RewordArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(
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

    let original_head = get_head_repo(&state.git.repo)?.to_string();

    let mut handle = SpinnerBuilder::new()
        .text("Gathering logs")
        .start();

    // mimicing the rebase flow, its pretty similar, the only major difference
    // is that we dont gen a plan, but only reword selected commits
    let (logs, trailing_commits) = match args.scope {
        RewordScope::Commit { ref hash } => {
            // we need the parent commit, to get the root since
            // from is exclusive
            let parent = find_parent_commit(&state.git.repo, &hash)?;

            let logs = get_logs(
                &state.git,
                true,
                false,
                0,
                true,
                Some(&parent.to_string()),
                Some(&hash),
                None,
            )?;

            let trails = trailing_commits(&state.git.repo, &hash)?;

            (logs, trails)
        }
        RewordScope::Last { count } => {
            let mut logs = get_logs(
                &state.git, true, false, count, false, None, None,
                None,
            )?;

            // if logs are reversed
            logs.git_logs
                .reverse();

            // return an empty vec for now
            // want to just leave trails empty
            // instead of returning an
            // option to unwrap
            (logs, Vec::new())
        }
        RewordScope::Range { ref from, ref to } => {
            let logs = get_logs(
                &state.git,
                true,
                false,
                0,
                true,
                Some(&from),
                to.as_deref(),
                None,
            )?;

            let to_hash = to
                .to_owned()
                .unwrap_or(
                    get_head_repo(&state.git.repo)?.to_string(),
                );

            let trails = trailing_commits(&state.git.repo, &to_hash)?;

            (logs, trails)
        }
    };

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

    handle.done();

    handle = SpinnerBuilder::new()
        .text("Generating request")
        .start();

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

    let schema =
        create_reword_schema(schema_settings, &state.settings)?;

    let request =
        create_reword_request(&state.settings, &state.git, &log_strs);

    handle.done();

    loop {
        let handle = SpinnerBuilder::new()
            .text("Generating commits")
            .start();

        let response: serde_json::Value = match extract_from_provider(
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

        let mut raw_commits =
            parse_to_reword_commit_schema(response)?;

        handle.done();

        print::commits::response_commits(&raw_commits, false)?;

        let mut regenerate = false;

        loop {
            let selected =
                Menu::new("What do you want to do?", &RESPONSE_OPTS)
                    .render()?;

            match selected {
                ResponseActions::Apply => {
                    let commit_messages: Vec<String> = raw_commits
                        .iter()
                        .cloned()
                        .map(|c| {
                            process_reword_commit_message(
                                c,
                                &state.settings,
                            )
                        })
                        .collect();

                    match apply(
                        &state.git,
                        &logs,
                        &commit_messages,
                        &trailing_commits,
                    ) {
                        // my god
                        Ok(retry) => {
                            if retry {
                                reset_repo_hard(
                                    &state.git.repo,
                                    &original_head,
                                )?;
                                continue;
                            }
                        }
                        Err(e) => {
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
                    raw_commits = edit_commits(&raw_commits)?;

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

fn apply(
    git: &GitRepo,
    logs: &Logs,
    new_commit_messages: &[String],
    trailing_commits: &[String],
) -> anyhow::Result<bool> {
    // for the range and everything to work
    // when applying, gonna need to quickly
    // mimic the reset flow from rebase
    // reset to -> parent of from commit
    let oldest = &logs.git_logs[0].commit_hash;

    let parent = find_parent_commit(&git.repo, oldest)?;

    reset_repo_hard(&git.repo, &parent.to_string())?;

    for (idx, log) in logs
        .git_logs
        .iter()
        .enumerate()
    {
        let commit = log
            .commit_hash
            .to_owned();

        if let Some(message) = new_commit_messages.get(idx) {
            cherry_pick_reword(&git.repo, &commit, message)?;
        } else {
            return Err(anyhow::anyhow!("bad index"));
        }
    }

    if !trailing_commits.is_empty() {
        cherry_pick_commits(&git.repo, trailing_commits)?;
    }
    // readd the trailing commits if any

    // then sync it
    force_checkout_head(&git.repo)?;

    Ok(false)
}
