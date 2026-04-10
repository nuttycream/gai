use serde_json::Value;

use crate::{
    args::{CommitArgs, GlobalArgs},
    git::{
        DiffStrategy, Diffs, GitRepo, StagingStrategy,
        StatusStrategy,
        commit::{GitCommit, apply_commits},
        diffs::get_diffs_from_statuses,
    },
    print::{
        self, menu::Menu, progressbar::SpinnerBuilder,
        renderer::Renderer, retry_prompt, style::StyleConfig,
    },
    providers::{extract_from_provider, provider::ProviderKind},
    requests::{Request, commit::create_commit_request},
    responses::commit::{parse_to_commit_schema, process_commit},
    schema::{SchemaSettings, commit::create_commit_response_schema},
    settings::Settings,
    state::State,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum ResponseActions {
    Apply,
    Regenerate,
    Edit,
    Response,
    Quit,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum EditWhat {
    Prefix,
    Scope,
    Breaking,
    Header,
    Body,
}

const RESPONSE_OPTS: [(ResponseActions, char, &str); 5] = [
    (ResponseActions::Apply, 'y', "apply all commit/s"),
    (ResponseActions::Regenerate, 'r', "regenerate commits"),
    (ResponseActions::Edit, 'e', "edit a commit"),
    (ResponseActions::Response, 'f', "view the full response"),
    (ResponseActions::Quit, 'q', "quit"),
];

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
            .text("Generating commits")
            .start(&renderer);

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

        let mut regenerate = false;

        loop {
            let selected = Menu::new("Apply all?", &RESPONSE_OPTS)
                .render(&renderer)?;

            match selected {
                ResponseActions::Apply => {
                    let git_commits: Vec<GitCommit> = raw_commits
                        .iter()
                        .cloned()
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
                            println!(
                                "Failed to Apply Commits: {}",
                                e
                            );

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
                    regenerate = true;
                    break;
                }
                ResponseActions::Edit => {
                    let commits: Vec<String> = raw_commits
                        .iter()
                        .map(|c| c.to_string())
                        .collect();

                    let commit_to_edit =
                        match print::input::fuzzy_to_num(
                            &renderer,
                            "Which commit? ",
                            &commits,
                        )? {
                            print::input::InputType::Text(_) => {}
                            print::input::InputType::Number(_) => {}
                            print::input::InputType::None => {
                                continue;
                            }
                        };
                }
                ResponseActions::Response => {
                    print::commits::full_response(
                        &renderer,
                        &raw_commits,
                        &diffs,
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
