use bpaf::{Parser, construct, long, short};
use serde_json::Value;

use crate::{
    git::{GitRepo, checkout::checkout_commit, log::get_logs},
    opts::Commands,
    print::{menu::Menu, spinner::SpinnerBuilder},
    providers::{extract_from_provider, provider::ProviderKind},
    requests::find::create_find_request,
    responses::find::parse_to_find_schema,
    schema::{SchemaSettings, find::create_find_schema},
    settings::Settings,
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

const FIND_DESC: &str = "\
Search through commit history using an LLM to locate a commit
matching a natural-language query. Commit logs (and optionally the
files and diffs they touch) are sent to the configured provider,
which returns the best match along with reasoning and a confidence
score. From the interactive menu the result can be checked out,
inspected in full, re-queried, or retried.";

#[derive(Debug, Clone, Default)]
pub struct FindArgs {
    count: usize,
    files: bool,
    diffs: bool,
    reverse: bool,
    range: Option<String>,
    since: Option<String>,
}

pub fn find() -> impl Parser<Commands> {
    let count = short('n')
        .long("count")
        .help("Maximum number of commits to consider from history")
        .argument::<usize>("N")
        .fallback(0);

    let files = short('f')
        .long("files")
        .help("Include the list of files touched by each commit in the context")
        .switch();

    let diffs = short('d')
        .long("diffs")
        .help("Include the diffs of each commit in the context (implies --files)")
        .switch();

    let reverse = short('R')
        .long("reverse")
        .help("Walk history from oldest to newest instead of newest to oldest")
        .switch();

    let range = long("range")
        .help("Restrict the search to a commit range, e.g. HEAD~20..HEAD")
        .argument::<String>("RANGE")
        .optional();

    let since = long("since")
        .help("Restrict the search to commits more recent than DATE, e.g. '2 weeks ago'")
        .argument::<String>("DATE")
        .optional();

    construct!(FindArgs {
        count,
        files,
        diffs,
        reverse,
        range,
        since,
    })
    .to_options()
    .descr(FIND_DESC)
    .command("find")
    .help("Find a commit by natural-language description using a LLM provider")
    .map(Commands::Find)
}

pub fn run(
    args: &FindArgs,
    settings: &Settings,
) -> anyhow::Result<()> {
    let count = args.count;

    let git = GitRepo::open(None)?;

    let logs = get_logs(
        &git,
        args.files,
        args.diffs,
        count,
        args.reverse,
        None,
        None,
        None,
    )?;

    let schema_settings =
        if matches!(settings.provider, ProviderKind::OpenAI) {
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

        let req = create_find_request(&settings, &log_strs, &q);

        /* if args.since.is_some() {
            println!("{}", req);
            break;
        } */

        let response: Value = match extract_from_provider(
            &settings.provider,
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
                checkout_commit(&git.repo, &log.commit_hash)?;
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
