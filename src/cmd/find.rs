use serde_json::Value;

use crate::{
    args::{FindArgs, GlobalArgs},
    git::{checkout::checkout_commit, log::get_logs},
    print::{
        InputHistory, find::print, input_prompt, loading,
        retry_prompt,
    },
    providers::{extract_from_provider, provider::ProviderKind},
    requests::find::create_find_request,
    responses::find::parse_to_find_schema,
    schema::{SchemaSettings, find::create_find_schema},
    state::State,
};

pub fn run(
    args: &FindArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(None, global)?;

    let count = args.number;

    let logs = get_logs(
        &state.git,
        args.files,
        args.diffs,
        count,
        args.reverse,
        args.from.as_deref(),
        args.to.as_deref(),
        args.since,
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

    let mut log_strs = Vec::new();

    for (idx, log) in logs
        .git_logs
        .iter()
        .enumerate()
    {
        if args.files {
            let files: String = log.files.join(",");

            let item = if args.diffs {
                let diffs = log
                    .diffs
                    .to_string();

                format!(
                    "CommitID:[{}]\nCommitMessage:{}\nFiles:{}\nDiffs:{}",
                    idx, log.raw, files, diffs,
                )
            } else {
                format!(
                    "CommitID:[{}]\nCommitMessage:{}\nFiles:{}",
                    idx, log.raw, files
                )
            };

            log_strs.push(item);
        } else {
            let item = format!(
                "CommitID:[{}]\nCommitMessage:{}",
                idx, log.raw
            );

            log_strs.push(item);
        }
    }

    let count = if logs.git_logs.len() > count {
        logs.git_logs.len() as u32
    } else {
        count as u32
    };

    let schema = create_find_schema(schema_settings, count)?;
    let mut history = InputHistory::default();

    let mut query = String::new();
    let mut should_retry = false;

    loop {
        let q = if should_retry {
            query.to_owned()
        } else {
            match input_prompt(
                "What is your query?",
                Some(&mut history),
            )? {
                Some(q) => {
                    query = q.to_owned();
                    q
                }
                None => break,
            }
        };

        let text =
            format!("Querying through your commits for \"{}\"", q);

        let req = create_find_request(&state.settings, &log_strs, &q);

        /* if args.since.is_some() {
            println!("{}", req);
            break;
        } */

        let loading = loading::Loading::new(&text, global.compact)?;

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
                    "Gai received an error from the provider:\n{:#}",
                    e
                );

                loading.stop_with_message(&msg);

                if retry_prompt(None)? {
                    continue;
                } else {
                    break;
                }
            }
        };

        let result = parse_to_find_schema(response)?;

        loading.stop();

        let log = logs.git_logs[result.commit_id as usize].to_owned();

        let reasoning = if args.reasoning {
            Some(
                result
                    .reasoning
                    .as_str(),
            )
        } else {
            None
        };

        let opt =
            print(&log, args.files, reasoning, result.confidence)?;

        match opt {
            0 => {
                println!("Checking out {}", log.commit_hash);
                checkout_commit(&state.git.repo, &log.commit_hash)?;
                break;
            }
            1 => {
                println!("Entering another query...");
                should_retry = false;
                continue;
            }
            2 => {
                println!("Retrying...");
                should_retry = true;
                continue;
            }
            _ => {
                println!("Exiting...");
                break;
            }
        }
    }

    Ok(())
}
