use owo_colors::OwoColorize;
use serde_json::Value;
use strum::{IntoEnumIterator, VariantNames};

use crate::{
    args::{CommitArgs, GlobalArgs},
    git::{
        DiffStrategy, Diffs, GitRepo, StagingStrategy,
        StatusStrategy,
        commit::{GitCommit, apply_commits},
        diffs::get_diffs_from_statuses,
        status::get_commit_stats,
    },
    print::{self, menu::Menu, spinner::SpinnerBuilder},
    providers::{extract_from_provider, provider::ProviderKind},
    requests::{Request, commit::create_commit_request},
    responses::commit::{parse_to_commit_schema, process_commit},
    schema::{
        SchemaSettings,
        commit::{
            CommitSchema, PrefixType, create_commit_response_schema,
        },
    },
    settings::Settings,
    state::State,
};

#[derive(Debug, Clone)]
pub enum ResponseActions {
    Apply,
    Regen,
    Edit,
    Quit,
}

#[derive(Debug, Clone)]
pub enum EditActions {
    Next,
    Previous,
    Remove,
    Prefix,
    Scope,
    Header,
    Body,
    Quit,
}

pub const RESPONSE_OPTS: [(ResponseActions, char, &str); 4] = [
    (ResponseActions::Apply, 'y', "apply all commit/s"),
    (ResponseActions::Regen, 'r', "regenerate commits"),
    (ResponseActions::Edit, 'e', "edit a commit"),
    (ResponseActions::Quit, 'q', "quit"),
];

pub const EDIT_OPTS: [(EditActions, char, &str); 8] = [
    (EditActions::Next, 'n', "next commit"),
    (EditActions::Previous, 'r', "return to previous commit"),
    (EditActions::Remove, 'd', "remove commit from list"),
    (EditActions::Prefix, 'p', "edit prefix"),
    (EditActions::Scope, 's', "edit scope"),
    (EditActions::Header, 'h', "edit header"),
    (EditActions::Body, 'b', "edit body in $EDITOR"),
    (EditActions::Quit, 'q', "quit"),
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

    print::status::provider_info(
        &state
            .settings
            .provider,
        &state
            .settings
            .providers,
    )?;

    let handle = SpinnerBuilder::new()
        .text("Generating request")
        .start();

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
            "Repository does not have any known changes."
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

    handle.done();

    run_commit(req, schema, state.settings, state.git, state.diffs)?;

    Ok(())
}

fn run_commit(
    req: Request,
    schema: Value,
    cfg: Settings,
    git: GitRepo,
    mut diffs: Diffs,
) -> anyhow::Result<()> {
    loop {
        let handle = SpinnerBuilder::new()
            .text("Generating commits")
            .start();

        let result: Value = match extract_from_provider(
            &cfg.provider,
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

        let mut raw_commits =
            parse_to_commit_schema(result, &cfg.staging_type)?;

        handle.done();

        print::commits::response_commits(
            &raw_commits,
            matches!(cfg.staging_type, StagingStrategy::Hunks),
        )?;

        let mut regenerate = false;

        loop {
            let selected =
                Menu::new("What do you want to do?", &RESPONSE_OPTS)
                    .render()?;

            match selected {
                ResponseActions::Apply => {
                    let git_commits: Vec<GitCommit> = raw_commits
                        .iter()
                        .cloned()
                        .map(|c| process_commit(c, &cfg))
                        .collect();

                    let oids = match apply_commits(
                        &git.repo,
                        &git_commits,
                        &mut diffs.files,
                        &cfg.staging_type,
                    ) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!(
                                "failed to apply commits:\n{e}",
                            );

                            break;
                        }
                    };

                    for (i, oid) in oids
                        .iter()
                        .enumerate()
                    {
                        let (
                            branch_name,
                            files_changed,
                            insertions,
                            deletions,
                        ) = get_commit_stats(&git.repo, &oid)?;

                        let commit_msg = git_commits[i]
                            .message
                            .to_owned();

                        print::commits::completed_commit(
                            &branch_name,
                            &oid,
                            &commit_msg,
                            files_changed,
                            insertions,
                            deletions,
                        )?;
                    }

                    break;
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
                            cfg.staging_type,
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

pub fn edit_commits(
    commits: &[CommitSchema]
) -> anyhow::Result<Vec<CommitSchema>> {
    let mut res = commits.to_vec();

    // for previous to work properly
    let mut i = 0;
    while i < res.len() {
        let mut edited = res[i].to_owned();

        loop {
            let msg = format!(
                "{}\n({}/{}) Edit what?",
                // sum
                edited
                    .to_string()
                    .lines()
                    .next()
                    .unwrap_or(""),
                i + 1,
                res.len(),
            );

            let action = Menu::new(&msg, &EDIT_OPTS).render()?;

            match action {
                EditActions::Next => {
                    res[i] = edited.to_owned();
                    i = i.saturating_add(1);

                    break;
                }
                EditActions::Previous => {
                    // im assuming the user wants
                    // to save the progress here and
                    // go back?
                    res[i] = edited.to_owned();

                    if i > 0 {
                        i = i.saturating_sub(1);
                    }
                    break;
                }
                EditActions::Remove => {
                    res.remove(i);
                    break;
                }
                EditActions::Prefix => {
                    loop {
                        let raw = print::input::prompt(&format!(
                            "type [{}]: ",
                            PrefixType::VARIANTS.join("/")
                        ))?;
                        if raw.is_empty() {
                            break;
                        } // empty = cancel

                        let trimmed = raw.trim();

                        match PrefixType::iter().find(|p| {
                            p.to_string()
                                .eq_ignore_ascii_case(trimmed)
                        }) {
                            Some(p) => {
                                edited.prefix = p;
                                break;
                            }
                            None => eprintln!(
                                "not a valid type, try again"
                            ),
                        }
                    }

                    res[i] = edited.to_owned();
                    continue;
                }
                EditActions::Scope => {
                    let raw = print::input::prompt(
                        "scope (empty to clear): ",
                    )?;

                    let trimmed = raw.trim();

                    edited.scope = if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    };

                    res[i] = edited.to_owned();
                    continue;
                }
                EditActions::Header => {
                    edited.header =
                        crate::utils::open::edit(&edited.header)?
                            .trim()
                            .to_string();

                    res[i] = edited.to_owned();

                    continue;
                }

                EditActions::Body => {
                    let current = edited
                        .body
                        .as_deref()
                        .unwrap_or("");

                    let new_body = crate::utils::open::edit(current)?;

                    let trimmed = new_body.trim();

                    edited.body = if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    };
                    res[i] = edited.to_owned();

                    continue;
                }

                EditActions::Quit => {
                    res[i] = edited.to_owned();
                    return Ok(res);
                }
            }
        }
    }

    Ok(res)
}
