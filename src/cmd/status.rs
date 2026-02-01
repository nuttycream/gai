use crate::{
    args::{GlobalArgs, StatusArgs},
    git::{
        DiffStrategy, StatusStrategy, diffs::get_diffs,
        status::get_status,
    },
    print::status,
    state::State,
};

pub fn run(
    args: &StatusArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(
        global
            .config
            .as_deref(),
        global,
    )?;

    // todo impl something for this
    // so we dont have to pass in two vectors
    // into print
    // likely gonna be handled within git::GitStatus
    let staged = get_status(&state.git.repo, &StatusStrategy::Stage)?;
    let working_dir =
        get_status(&state.git.repo, &StatusStrategy::WorkingDir)?;

    status::print(
        &staged.branch_name,
        &staged.statuses,
        &working_dir.statuses,
        global.compact,
    )?;

    if args.verbose {
        let mut diff_strategy = DiffStrategy {
            ..Default::default()
        };

        if let Some(ref files_to_truncate) = state
            .settings
            .context
            .truncate_files
        {
            diff_strategy.truncated_files =
                files_to_truncate.to_owned();
        }

        if let Some(ref files_to_ignore) = state
            .settings
            .context
            .ignore_files
        {
            diff_strategy.ignored_files = files_to_ignore.to_owned();
        }

        let diffs = get_diffs(&state.git, &diff_strategy)?;

        println!("{}", diffs);
    }

    Ok(())
}
