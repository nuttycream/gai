use std::collections::HashMap;

use console::style;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use serde_json::Value;

use crate::{
    args::{CommitArgs, GlobalArgs},
    git::{
        DiffStrategy, Diffs, GitRepo, StagingStrategy,
        StatusStrategy,
        commit::{GitCommit, commit},
        diffs::{
            FileDiff, HunkId, find_file_hunks, get_diffs,
            remove_hunks,
        },
        staging::{stage_all, stage_file, stage_hunks},
    },
    print::{commits, loading::Loading},
    providers::{extract_from_provider, provider::ProviderKind},
    requests::{Request, commit::create_commit_request},
    responses::commit::{parse_from_schema, process_commit},
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

    state.diffs = get_diffs(&state.git, &diff_strategy)?;

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
            parse_from_schema(result, &cfg.staging_type)?;

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
                if apply_commits(
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
            if apply_commits(
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

fn apply_commits(
    repo: &GitRepo,
    git_commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_stragey: &StagingStrategy,
) -> bool {
    println!("Applying Commits...");
    match apply(repo, git_commits, og_file_diffs, staging_stragey) {
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

fn apply(
    git: &GitRepo,
    git_commits: &[GitCommit],
    og_file_diffs: &mut Vec<FileDiff>,
    staging_stragey: &StagingStrategy,
) -> anyhow::Result<()> {
    //todo when we implement verbose logging
    // make sure we log the files, hunks etc
    // before we apply commits

    for git_commit in git_commits {
        match staging_stragey {
            StagingStrategy::AllFilesOneCommit => {
                stage_all(&git.repo, ".")?;
                og_file_diffs.clear();
                commit(&git.repo, git_commit)?;

                // return early
                return Ok(());
            }
            StagingStrategy::AtomicCommits
            | StagingStrategy::OneFilePerCommit => {
                for file in &git_commit.files {
                    stage_file(&git.repo, file)?;
                    // remove if status matches
                    //remove_file(&git.repo, file)?;
                    og_file_diffs.retain(|f| f.path != file.as_str());
                }
            }
            StagingStrategy::Hunks => {
                // this commit should define its hunkids
                // to stage like:
                // commit 1: src/main.rs:0, src/main.rs:1 etc
                // group hunks based on the file paths
                // iterate over each file
                // find what hunks to stage
                // pass it into stage_hunks
                // stage_hunks should be able to apply
                // only the hunks it gets from here

                // file_path and a list of hunk indecises
                let mut files: HashMap<String, Vec<usize>> =
                    HashMap::new();

                // group hunks to their file_paths
                for hunk in &git_commit.hunk_ids {
                    let hunk_id = HunkId::try_from(hunk.as_str())?;
                    files
                        .entry(hunk_id.path.clone())
                        .or_default()
                        .push(hunk_id.index);
                }

                // now process each file
                for (file_path, hunk_ids) in files {
                    // find the original file associated
                    // with this from the og database
                    let og_file_diff = og_file_diffs
                        .iter()
                        .find(|f| f.path == file_path)
                        .ok_or({
                            anyhow::anyhow!(
                                "{} is not in the og_file_diffs",
                                file_path
                            )
                        })?;

                    if og_file_diff.untracked {
                        stage_file(&git.repo, &file_path)?;
                        og_file_diffs.retain(|f| f.path != file_path);
                        continue;
                    }

                    // get relevant hunk ids
                    let hunks =
                        find_file_hunks(og_file_diff, hunk_ids)?;

                    // stage hunks relevant to this file ONLY
                    let used =
                        stage_hunks(&git.repo, &file_path, &hunks)?;

                    remove_hunks(og_file_diffs, &file_path, &used);
                }
            }
        }

        commit(&git.repo, git_commit)?;
    }

    for file in og_file_diffs {
        for hunk in &file.hunks {
            println!("hunk [{}:{}] not applied", file.path, hunk.id);
        }
    }

    Ok(())
}
