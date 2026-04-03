use serde_json::Value;
use strum::VariantNames;

use crate::{
    args::{CommitArgs, GlobalArgs},
    git::{
        DiffStrategy, Diffs, GitRepo, StagingStrategy,
        StatusStrategy,
        commit::{GitCommit, apply_commits},
        diffs::get_diffs_from_statuses,
    },
    print::{
        self, menu::MenuChosenOption, progressbar::SpinnerBuilder,
        renderer::Renderer, retry_prompt, style::StyleConfig,
    },
    providers::{extract_from_provider, provider::ProviderKind},
    requests::{Request, commit::create_commit_request},
    responses::commit::{parse_to_commit_schema, process_commit},
    schema::{SchemaSettings, commit::create_commit_response_schema},
    settings::Settings,
    state::State,
};

#[derive(Debug, VariantNames, strum::FromRepr)]
#[strum(serialize_all = "lowercase")]
enum ResponseActions {
    Apply,
    Regenerate,
    Edit,
    #[strum(serialize = "full view")]
    ViewResponse,
    Exit,
}

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

    let renderer =
        Renderer::new(StyleConfig::default(), global.compact)?;

    if args.staged {
        state
            .settings
            .commit
            .only_staged = true;
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
        println!("Repository does not have any known changes.");
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
        renderer,
        state.settings,
        state.git,
        state.diffs,
    )?;

    Ok(())
}

fn run_commit(
    req: Request,
    schema: Value,
    renderer: Renderer,
    cfg: Settings,
    git: GitRepo,
    mut diffs: Diffs,
) -> anyhow::Result<()> {
    print::status::provider_info(
        &renderer,
        &cfg.provider,
        &cfg.providers,
    )?;

    loop {
        let handle = SpinnerBuilder::new()
            .text("Generating Commits")
            .start();

        let result: Value = match extract_from_provider(
            &cfg.provider,
            req.to_owned(),
            schema.to_owned(),
        ) {
            Ok(r) => r,
            Err(e) => {
                let err = format!(
                    "Done but Gai received an error from the provider: {:#}",
                    e
                );

                handle.text(err);
                handle.error();

                if retry_prompt(None)? {
                    continue;
                } else {
                    break;
                }
            }
        };

        let raw_commits =
            parse_to_commit_schema(result, &cfg.staging_type)?;

        let msg = format!(
            "Done! Received {} Commit{}",
            raw_commits.len(),
            if raw_commits.len() == 1 { "" } else { "s" }
        );

        handle.text(msg);
        handle.stop();

        print::commits::response_commits(
            &renderer,
            &raw_commits,
            matches!(cfg.staging_type, StagingStrategy::Hunks),
        )?;

        let selected = match print::menu::inline_menu(
            &renderer,
            "What do you want to do?",
            ResponseActions::VARIANTS,
        )? {
            MenuChosenOption::Selected(i) => {
                ResponseActions::from_repr(i)
                    .expect("uhh, somehow didn't get the correct idx")
            }
            MenuChosenOption::Cancelled => break,
        };

        match selected {
            ResponseActions::Apply => {
                let git_commits: Vec<GitCommit> = raw_commits
                    .into_iter()
                    .map(|c| process_commit(c, &cfg))
                    .collect();

                match apply_commits(
                    &git.repo,
                    &git_commits,
                    &mut diffs.files,
                    &cfg.staging_type,
                ) {
                    Ok(_) => break,
                    Err(e) => {
                        println!("Failed to Apply Commits: {}", e);

                        if retry_prompt(None).unwrap() {
                            println!("Regenerating...");
                            continue;
                        } else {
                            println!("Exiting");
                            break;
                        }
                    }
                }
            }
            ResponseActions::Regenerate => {
                println!("Regenerating");
                continue;
            }
            ResponseActions::Edit => {
                todo!()
            }
            ResponseActions::ViewResponse => {
                todo!()
            }
            ResponseActions::Exit => {
                println!("Exiting")
            }
        }

        break;
    }

    Ok(())
}
