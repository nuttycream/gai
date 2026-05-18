use bpaf::{Parser, construct, short};

use crate::{
    git::{
        DiffStrategy, GitRepo, StatusStrategy,
        diffs::get_diffs_from_statuses, status::get_status,
    },
    opts::Commands,
    print::status,
    requests::tokens::estimate_token_count,
    settings::Settings,
};

const STATUS_DESC: &str = "\
Show a summary of the current repository state, the active provider
configuration, and an estimated token count for each file in the
working tree. Useful for previewing what context would be sent to
the LLM provider and for spotting files that should
be ignored or truncated via the configuration.";

#[derive(Debug, Clone)]
pub struct StatusArgs {
    pub staged: bool,
    pub tokens: bool,
}

pub fn status() -> impl Parser<Commands> {
    let staged = short('s')
        .long("staged")
        .help("Only consider staged changes")
        .switch();

    let tokens = short('t')
        .long("tokens")
        .help("Only print the estimated token list per file")
        .switch();

    construct!(StatusArgs { staged, tokens })
        .to_options()
        .descr(STATUS_DESC)
        .command("status")
        .help("Show repo status, provider info, and token estimates")
        .map(Commands::Status)
}

pub fn run(
    args: &StatusArgs,
    settings: &Settings,
) -> anyhow::Result<()> {
    let git = GitRepo::open(None)?;

    let status_strategy = if args.staged
        || settings
            .commit
            .only_staged
    {
        StatusStrategy::Stage
    } else {
        StatusStrategy::default()
    };

    if !args.tokens {
        // todo impl something for this
        // so we dont have to pass in two vectors
        // into print
        // likely gonna be handled within git::GitStatus
        let staged = get_status(&git.repo, &status_strategy)?;
        let working_dir = get_status(&git.repo, &status_strategy)?;

        let provider = settings.provider;

        status::provider_info(&provider, &settings.providers)?;

        status::repo_status(
            &staged.branch_name,
            &staged.statuses,
            &working_dir.statuses,
        )?;
    }

    let diff_strategy = DiffStrategy {
        status_strategy,
        ..Default::default()
    };

    let diffs = get_diffs_from_statuses(
        &git.repo,
        &git.workdir,
        &diff_strategy,
    )?;

    let file_tokens: Vec<(String, usize)> = diffs
        .files
        .iter()
        .map(|file| {
            let mut txt = String::new();

            // mimicing what gets sent to the prompt
            // ideally, this is done near the request
            for hunk in &file.hunks {
                txt.push_str(&format!(
                    "HunkId[{}:{}]\n",
                    file.path, hunk.id
                ));

                for line in &hunk.lines {
                    txt.push_str(&format!(
                        "{}{}\n",
                        line.line_type, line.content
                    ));
                }
            }

            (file.path.clone(), estimate_token_count(&txt) as usize)
        })
        .collect();

    let mut total: usize = 0;

    for (path, tokens) in &file_tokens {
        total = total.saturating_add(*tokens);
        // temp println
        // TODO: remove, use status::repo_status
        println!("file:{} tokens:{}", path, tokens);
    }

    Ok(())
}
