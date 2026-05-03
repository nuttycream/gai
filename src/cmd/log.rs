use crate::{
    args::{GlobalArgs, LogArgs},
    git::log::get_logs,
    state::State,
};

pub fn run(
    args: &LogArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(None, global)?;

    let count = args
        .number
        .unwrap_or_default();

    let _logs = get_logs(
        &state.git,
        true,
        false,
        count,
        args.reverse,
        None,
        None,
        None,
    )?;

    Ok(())
}
