use serde_json::Value;

use crate::{
    args::{FindArgs, GlobalArgs},
    git::{checkout::checkout_commit, log::get_logs},
    print::{menu::Menu, spinner::SpinnerBuilder},
    providers::{extract_from_provider, provider::ProviderKind},
    requests::find::create_find_request,
    responses::find::parse_to_find_schema,
    schema::{SchemaSettings, find::create_find_schema},
    state::State,
};

#[derive(Debug, Clone)]
enum ResponseActions {
    Checkout,
    ReQuery,
    Full,
    Retry,
    Quit,
}

const RESPONSE_OPTS: [(ResponseActions, char, &str); 5] = [
    (ResponseActions::Checkout, 'c', "checkout the commit"),
    (ResponseActions::Full, 'f', "see full commit information"),
    (ResponseActions::ReQuery, 'a', "retry with another query"),
    (ResponseActions::Retry, 'r', "retry with the same query"),
    (ResponseActions::Quit, 'q', "quit"),
];

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

    let query = String::new();
    let mut should_retry = false;

    loop {
        let q = if should_retry {
            query.to_owned()
        } else {
            crate::print::input::prompt(
                "What do you want to search for? ",
            )?
        };

        let handle = SpinnerBuilder::new()
            .text("Searching through commits")
            .start();

        let req = create_find_request(&state.settings, &log_strs, &q);

        /* if args.since.is_some() {
            println!("{}", req);
            break;
        } */

        let response: Value = match extract_from_provider(
            &state
                .settings
                .provider,
            req.to_owned(),
            schema.to_owned(),
        ) {
            Ok(r) => r,
            Err(e) => {
                handle.error();

                eprintln!("error from the provider:\n{:#}", e);

                break;
            }
        };

        let result = parse_to_find_schema(response)?;

        handle.done();

        let log = logs.git_logs[result.commit_id as usize].to_owned();

        let reasoning = result
            .reasoning
            .as_str();

        crate::print::find::found_commit(
            &log,
            reasoning,
            result.confidence,
        )?;

        match Menu::new("What do you want to do? ", &RESPONSE_OPTS)
            .render()?
        {
            ResponseActions::Checkout => {
                checkout_commit(&state.git.repo, &log.commit_hash)?;
                break;
            }
            ResponseActions::ReQuery => {
                should_retry = false;
                continue;
            }
            ResponseActions::Full => todo!(),
            ResponseActions::Retry => {
                should_retry = true;
                continue;
            }
            ResponseActions::Quit => break,
        }
    }

    Ok(())
}
