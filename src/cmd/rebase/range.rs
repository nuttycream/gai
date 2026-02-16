use console::style;
use git2::Oid;

use crate::{
    git::{
        GitRepo,
        commit::find_parent_commit,
        log::{get_logs, get_short_hash},
        rebase::trailing_commits,
        utils::get_head_repo,
    },
    print::log::print_logs,
};

/// stores additional range info
/// such as trailing commits and to hash
/// should this be used in all rebase
/// workflows?
pub(super) struct RebaseRange {
    pub from: Oid,
    pub to: Option<Oid>,
    pub trailing: Option<Vec<Oid>>,
}

pub(super) fn rebase_range(
    repo: &GitRepo,
    from_hash: Option<&str>,
    to_hash: Option<&str>,
    interactive: bool,
) -> anyhow::Result<Option<RebaseRange>> {
    if interactive {
        return specify_range_flow(repo);
    }

    let from = from_hash.unwrap();

    let logs = get_logs(
        repo,
        false,
        false,
        0,
        false,
        Some(from),
        to_hash,
        None,
    )?;

    let count = logs.git_logs.len();

    let oid = find_parent_commit(&repo.repo, from)?;

    println!(
        "{} Rebasing {} commit{} from {}",
        style("→").green(),
        style(count).cyan(),
        if count == 1 { "" } else { "s" },
        //get_short_hash()
        style(&from[..from.len().min(7)]).dim()
    );

    if let Some(to) = to_hash {
        let trailing = trailing_commits(&repo.repo, to)?;

        return Ok(Some(RebaseRange {
            from: oid,
            to: Some(Oid::from_str(to)?),
            trailing: Some(trailing),
        }));
    }

    let to = get_head_repo(&repo.repo)?;

    Ok(Some(RebaseRange {
        from: oid,
        to: Some(to),
        trailing: None,
    }))
}

fn specify_range_flow(
    repo: &GitRepo
) -> anyhow::Result<Option<RebaseRange>> {
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

        return Ok(Some(RebaseRange {
            from: diverge_from,
            to: todo!(),
            trailing: todo!(),
        }));
    }
}
