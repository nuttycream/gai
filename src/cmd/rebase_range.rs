use console::style;
use git2::Oid;

use crate::{
    git::{
        GitRepo,
        commit::find_parent_commit,
        log::{get_logs, get_short_hash},
    },
    print::log::print_logs,
};

pub(super) fn rebase_range(
    repo: &GitRepo,
    from_hash: Option<&str>,
    interactive: bool,
) -> anyhow::Result<Option<Oid>> {
    if interactive {
        return specify_range_flow(repo);
    }

    if let Some(from_hash) = from_hash {
        let logs = get_logs(
            repo,
            false,
            false,
            0,
            false,
            Some(from_hash),
            None,
            None,
        )?;

        let count = logs.git_logs.len();

        let oid = find_parent_commit(&repo.repo, from_hash)?;

        println!(
            "{} Rebasing {} commit{} from {}",
            style("→").green(),
            style(count).cyan(),
            if count == 1 { "" } else { "s" },
            //get_short_hash()
            style(
                &from_hash[..from_hash
                    .len()
                    .min(7)]
            )
            .dim()
        );

        Ok(Some(oid))
    } else {
        return Ok(None);
    }
}

fn specify_range_flow(repo: &GitRepo) -> anyhow::Result<Option<Oid>> {
    let logs =
        get_logs(repo, false, false, 0, false, None, None, None)?;

    if logs
        .git_logs
        .is_empty()
    {
        println!("No commits found. Exiting...");
        return Ok(None);
    }

    loop {
        // logs are ordered newwest, so we use
        // older and newer terms
        // to avoid confusion with list position
        let first = match print_logs(
            &logs.git_logs,
            Some("Select the starting range"),
            Some(10),
        )? {
            Some(s) => s,
            None => {
                println!("Exiting...");
                return Ok(None);
            }
        };

        let commit = &logs.git_logs[first];

        let logs = get_logs(
            repo,
            false,
            false,
            0,
            false,
            Some(&commit.commit_hash),
            None,
            None,
        )?;

        let count = logs.git_logs.len();

        if count == 0 {
            println!(
                "No commits in selected range OR commit selected is HEAD. Resetting..."
            );
            continue;
        }

        println!(
            "{} Rebasing {} commit{} since {}:",
            style("→").green(),
            style(count).cyan(),
            if count == 1 { "" } else { "s" },
            style("HEAD").red(),
        );

        println!(
            " From: {} {}",
            style(&get_short_hash(commit)).dim(),
            String::from(commit.to_owned())
        );

        let diverge_from =
            find_parent_commit(&repo.repo, &commit.commit_hash)?;

        return Ok(Some(diverge_from));
    }
}
