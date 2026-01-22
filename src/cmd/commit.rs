use console::style;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use serde_json::Value;

use crate::{
    args::{CommitArgs, GlobalArgs},
    git::{
        DiffStrategy, Diffs, GitRepo, StagingStrategy,
        StatusStrategy,
        commit::{GitCommit, apply_commits},
        diffs::{FileDiff, get_diffs_from_statuses},
    },
    print::{commits, loading::Loading},
    providers::{extract_from_provider, provider::ProviderKind},
    requests::{Request, commit::create_commit_request},
    responses::commit::{parse_to_commit_schema, process_commit},
    schema::{SchemaSettings, commit::create_commit_response_schema},
    settings::Settings,
    state::State,
};

pub fn run(
    args: &CommitArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let mut state = State::new(
        global
            .config
            .as_deref(),
        global,
    )?;

    state
        .settings
        .prompt
        .hint = global
        .hint
        .to_owned();

    if args.staged {
        state
            .settings
            .commit
            .only_staged = true;
    }

    if let Some(provider) = global.provider {
        state
            .settings
            .provider = provider;
    }

    let status_strategy = if state
        .settings
        .commit
        .only_staged
    {
        StatusStrategy::Stage
    } else {
        StatusStrategy::default()
    };

    let mut diff_strategy = DiffStrategy {
        status_strategy,
        ..Default::default()
    };

    if let Some(ref files_to_truncate) = state
        .settings
        .context
        .truncate_files
    {
        diff_strategy.truncated_files = files_to_truncate.to_owned();
    }

    if let Some(ref files_to_ignore) = state
        .settings
        .context
        .ignore_files
    {
        diff_strategy.ignored_files = files_to_ignore.to_owned();
    }

    state.diffs = get_diffs_from_statuses(
        &state.git.repo,
        &state.git.workdir,
        &diff_strategy,
    )?;

    if state
        .diffs
        .files
        .is_empty()
    {
        println!(
            "{}",
            style("Repository does not have any known changes.")
                .yellow()
                .bold()
        );
        return Ok(());
    }

    // openai seems like the only one that needs this
    let schema_settings = if matches!(
        state
            .settings
            .provider,
        ProviderKind::OpenAI
    ) {
        SchemaSettings::default().additional_properties(false)
    } else {
        SchemaSettings::default()
    };

    let schema = create_commit_response_schema(
        schema_settings,
        &state.settings,
        &state
            .diffs
            .as_files(),
        &state
            .diffs
            .as_hunks(),
    )?;

    let req = create_commit_request(
        &state.settings,
        &state.git,
        &state
            .diffs
            .to_string(),
    );

    /* println!("{}", serde_json::to_string_pretty(&schema)?);
    println!("{:#?}", req); */

    run_commit(
        req,
        schema,
        state.settings,
        state.git,
        state.diffs,
        args.skip_confirmation,
        global.compact,
    )?;

    Ok(())
}

fn run_commit(
    req: Request,
    schema: Value,
    cfg: Settings,
    git: GitRepo,
    mut diffs: Diffs,
    skip_confirmation: bool,
    compact: bool,
) -> anyhow::Result<()> {
    let provider_display = format!(
        "Generating Commits Using {}({})",
        style(&cfg.provider).blue(),
        style(
            cfg.providers
                .get_model(&cfg.provider)
        )
        .dim()
    );

    loop {
        let loading = Loading::new(&provider_display, compact)?;

        loading.start();

        let result: Value = match extract_from_provider(
            &cfg.provider,
            req.to_owned(),
            schema.to_owned(),
        ) {
            Ok(r) => r,
            Err(e) => {
                loading.stop();
                println!(
                    "Done but Gai received an error from the provider: {:#}",
                    e
                );

                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Retry?")
                    .interact()?
                {
                    continue;
                } else {
                    break;
                }
            }
        };

        let raw_commits =
            parse_to_commit_schema(result, &cfg.staging_type)?;

        loading.stop();

        println!(
            "Done! Received {} Commit{}",
            raw_commits.len(),
            if raw_commits.len() == 1 { "" } else { "s" }
        );

        let selected = commits::print_response_commits(
            &raw_commits,
            compact,
            matches!(cfg.staging_type, StagingStrategy::Hunks),
            skip_confirmation,
        )?;

        let git_commits: Vec<GitCommit> = raw_commits
            .into_iter()
            .map(|c| process_commit(c, &cfg))
            .collect();

        let selected = match selected {
            Some(s) => s,
            None => {
                if apply(
                    &git,
                    &git_commits,
                    &mut diffs.files,
                    &cfg.staging_type,
                ) {
                    continue;
                }
                0
            }
        };

        if selected == 0 {
            if apply(
                &git,
                &git_commits,
                &mut diffs.files,
                &cfg.staging_type,
            ) {
                continue;
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
    repo: &GitRepo,
    git_commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_stragey: &StagingStrategy,
) -> bool {
    println!("Applying Commits...");
    match apply_commits(
        &repo.repo,
        git_commits,
        og_file_diffs,
        staging_stragey,
    ) {
        Ok(_) => false,
        Err(e) => {
            println!("Failed to Apply Commits: {}", e);

            let options = ["Retry", "Exit"];
            let selection =
                Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select an option:")
                    .items(options)
                    .default(0)
                    .interact()
                    .unwrap();

            if selection == 0 {
                println!("Regenerating...");
                true
            } else {
                println!("Exiting");
                false
            }
        }
    }
}
