use crate::{
    args::{GlobalArgs, LogArgs},
    git::log::get_logs,
    print::log,
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

    let logs = get_logs(
        &state.git,
        true,
        false,
        count,
        args.reverse,
        None,
        None,
        None,
    )?;

    match log::print_logs(&logs.git_logs, None, None)? {
        Some(s) => {
            // todo impl perform checkout
            let log: String = logs.git_logs[s]
                .to_owned()
                .into();
            println!("Selected: {}", log);
        }
        None => {
            // do nothing
        }
    }

    Ok(())
}
