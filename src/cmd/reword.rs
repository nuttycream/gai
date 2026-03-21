use crate::{
    args::{GlobalArgs, RewordArgs},
    git::{
        GitRepo,
        checkout::force_checkout_head,
        commit::find_parent_commit,
        log::{Logs, get_log, get_logs},
        rebase::{
            cherry_pick_commits, cherry_pick_reword, trailing_commits,
        },
        reset::reset_repo_hard,
        status::is_workdir_clean,
        utils::get_head_repo,
    },
    print::{
        commits::print_response_commits, loading, print_retry_prompt,
    },
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

    // mimicing the rebase flow, its pretty similar, the only major difference
    // is that we dont gen a plan, but only reword selected commits
    let (logs, trailing_commits) = if let Some(ref hash) = args.commit
    {
        let logs = get_logs(
            &state.git,
            true,
            false,
            0,
            true,
            Some(hash),
            Some(hash),
            None,
        )?;

        let trails = trailing_commits(&state.git.repo, hash)?;

        (logs, trails)
    } else if let Some(last_n) = args.last {
        let logs = get_logs(
            &state.git, true, false, last_n, true, None, None, None,
        )?;

        // return an empty vec for now
        // want to just leave trails empty
        // instead of returning an
        // option to unwrap
        (logs, Vec::new())
    } else if args.from.is_some() {
        let logs = get_logs(
            &state.git,
            true,
            false,
            0,
            true,
            args.from.as_deref(),
            args.to.as_deref(),
            None,
        )?;

        let to_hash = args
            .to
            .to_owned()
            .unwrap_or(get_head_repo(&state.git.repo)?.to_string());

        let trails = trailing_commits(&state.git.repo, &to_hash)?;

        (logs, trails)
    } else {
        // do interactive
        todo!()
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

    loop {
        let loading = loading::Loading::new(
            "Generating New Commit Messages",
            global.compact,
        )?;

        loading.start();

        let response: serde_json::Value = match extract_from_provider(
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

        let raw_commits = parse_to_reword_commit_schema(response)?;

        loading.stop();

        println!(
            "Done! Received {} Commit{}",
            raw_commits.len(),
            if raw_commits.len() == 1 { "" } else { "s" }
        );

        let selected = if let Some(s) = print_response_commits(
            &raw_commits,
            global.compact,
            false,
            false,
        )? {
            s
        } else {
            return Ok(());
        };

        let commit_messages: Vec<String> = raw_commits
            .into_iter()
            .map(|c| {
                process_reword_commit_message(c, &state.settings)
            })
            .collect();

        if selected == 0 {
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
                    reset_repo_hard(&state.git.repo, &original_head)?;
                    return Err(e);
                }
            }
        } else if selected == 1 {
            println!("Regenerating");
            continue;
        } else if selected == 2 {
            println!("Exiting");
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

    let parent = find_parent_commit(&git.repo, &oldest)?;

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
            cherry_pick_reword(&git.repo, &commit, &message)?;
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
