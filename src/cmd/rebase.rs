use console::style;

use crate::{
    args::{GlobalArgs, RebaseArgs},
    git::{
        branch::{find_divergence_branch, validate_branch_exists},
        log::get_logs,
    },
    state::State,
};

pub fn run(
    args: &RebaseArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    // get from branch name
    // get onto branch , defaults to head
    // get list of commits from the branch
    // maybe with get_logs()?
    //
    // collect diffs from commits
    //
    // should we send as logs?
    // or as a giant diff?
    //
    // if a giant diff, then we can
    // reuse commitschema
    // to generate a list of commits
    // to apply to onto
    //
    // if send as logs, how is that going
    // to be handled, should we create
    // a schema, and just edit the commit messages
    // from logs?

    // create the request
    // send the request + schema
    // parse response
    // prompt the user
    // to rebase on top as commits
    // or merge commits?

    let state = State::new(
        global
            .config
            .as_deref(),
    )?;

    if !validate_branch_exists(&state.git.repo, &args.branch)? {
        println!(
            "Branch {}, {}",
            style(&args.branch).bold(),
            style("does not exist or is an invalid branch name")
                .red()
        );

        return Ok(());
    }

    if let Some(onto) = &args.onto
        && !validate_branch_exists(&state.git.repo, onto)?
    {
        println!(
            "Branch {}, {}",
            style(&onto).bold(),
            style("does not exist or is an invalid branch name")
                .red()
        );

        return Ok(());
    }

    let commit =
        find_divergence_branch(&state.git.repo, &args.branch)?
            .to_string();

    // collected logs from diverging branch
    let logs = get_logs(
        &state.git,
        true,
        false,
        0,
        false,
        Some(&commit),
        None,
        None,
    )?;

    println!("{:#?}", logs);

    Ok(())
}
