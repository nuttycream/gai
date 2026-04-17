use serde_json::Value;

use crate::{
    args::{CommitArgs, GlobalArgs},
    git::{
        DiffStrategy, Diffs, GitRepo, StagingStrategy,
        StatusStrategy,
        commit::{GitCommit, apply_commits},
        diffs::get_diffs_from_statuses,
        status::get_commit_stats,
    },
    print::{
        self, menu::Menu, renderer::Renderer,
        spinner::SpinnerBuilder, style::StyleConfig,
    },
    providers::{extract_from_provider, provider::ProviderKind},
    requests::{Request, commit::create_commit_request},
    responses::commit::{parse_to_commit_schema, process_commit},
    schema::{
        SchemaSettings,
        commit::{CommitSchema, create_commit_response_schema},
    },
    settings::Settings,
    state::State,
};

#[derive(Debug, Clone)]
enum ResponseActions {
    Apply,
    Regenerate,
    Edit,
    Quit,
}

#[derive(Debug, Clone)]
enum EditActions {
    Next,
    Previous,
    Prefix,
    Scope,
    Header,
    Body,
    Quit,
}

const RESPONSE_OPTS: [(ResponseActions, char, &str); 4] = [
    (ResponseActions::Apply, 'y', "apply all commit/s"),
    (ResponseActions::Regenerate, 'r', "regenerate commits"),
    (ResponseActions::Edit, 'e', "edit a commit"),
    (ResponseActions::Quit, 'q', "quit"),
];

const EDIT_OPTS: [(EditActions, char, &str); 7] = [
    (EditActions::Next, 'n', "next commit"),
    (EditActions::Previous, 'r', "return to previous commit"),
    (EditActions::Prefix, 'p', "select a new prefix"),
    (EditActions::Scope, 's', "edit the scope message"),
    (EditActions::Header, 'h', "edit the header in $EDITOR"),
    (EditActions::Body, 'b', "edit the body in $EDITOR"),
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
    print::status::provider_info(&cfg.provider, &cfg.providers)?;

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

                        let commit_msg =
                            raw_commits[i].just_the_header();

                        print::commits::completed_commit(
                            &renderer,
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
                ResponseActions::Regenerate => {
                    regenerate = true;
                    break;
                }
                ResponseActions::Edit => {
                    let edited =
                        edit_commits(&renderer, &raw_commits)?;

                    if !edited.is_empty() {
                        raw_commits = edited;

                        print::commits::response_commits(
                            &renderer,
                            &raw_commits,
                            matches!(
                                cfg.staging_type,
                                StagingStrategy::Hunks
                            ),
                        )?;
                    }

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

fn edit_commits(
    renderer: &Renderer,
    commits: &[CommitSchema],
) -> anyhow::Result<Vec<CommitSchema>> {
    let mut res: Vec<CommitSchema> = commits.to_vec();

    // for previous to work properly
    let mut i = 0;
    while i < res.len() {
        let mut edited = res[i].to_owned();

        let msg = format!(
            "{}\n({}/{}) Edit what?",
            edited.just_the_header(),
            i + 1,
            commits.len(),
        );

        loop {
            let action =
                Menu::new(&msg, &EDIT_OPTS).render(renderer)?;

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
                EditActions::Prefix => {
                    // grab variants and have user fuzzy find them
                    // i feel this is decent ux, if its atrocious
                    // we can change it
                    todo!();
                }
                EditActions::Scope => {
                    edited.scope = Some(print::input::prompt(
                        renderer, "scope: ",
                    )?);

                    continue;
                }
                EditActions::Header => {
                    let new_header =
                        crate::utils::open::edit(&edited.header)?;

                    edited.header = new_header
                        .trim()
                        .to_string();

                    continue;
                }
                EditActions::Body => {
                    let body_text = edited
                        .body
                        .as_deref()
                        .unwrap_or("");

                    let new_body =
                        crate::utils::open::edit(body_text)?;

                    edited.body = Some(
                        new_body
                            .trim()
                            .to_string(),
                    );

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
