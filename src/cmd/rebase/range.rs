use git2::Oid;

use crate::git::{
    GitRepo,
    commit::find_parent_commit,
    log::{get_logs, get_short_hash},
    rebase::trailing_commits,
    utils::get_head_repo,
};

/// stores additional range info
/// such as trailing commits and to hash
/// should this be used in all rebase
/// workflows?
pub(super) struct RebaseRange {
    pub from: Oid,
    pub to: Option<String>,
    pub trailing: Option<Vec<String>>,
}

/// rebase_range will act differently from the other
/// types of gai rebases. it'll "optionally" return an
/// accompanying to commit and trailing_commits.
/// trailing_commits are commits that come after the TO commit
/// this is so that when we apply the generated diffs, we can
/// cherry-pick the trailing_commits back in.
/// to do this effectively we first have to get the
/// proper diff range
/// and set the repo up to allow us to
/// cherry pick back those commits in,
/// this is not done in this function
/// but by the main rebase_run caller
/// ideally, we reset hard to the TO commit (this will get rid of
/// the changes), then from the TO commit, we do a MIXED reset
/// to the FROM commit, effectively gathering the
/// necessary diff for the LLM
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
        "→",
        count,
        if count == 1 { "" } else { "s" },
        //get_short_hash()
        &from[..from.len().min(7)]
    );

    if let Some(to) = to_hash {
        let trailing = trailing_commits(&repo.repo, to)?;

        return Ok(Some(RebaseRange {
            from: oid,
            to: Some(to.to_string()),
            trailing: Some(trailing),
        }));
    }

    let to = get_head_repo(&repo.repo)?;

    Ok(Some(RebaseRange {
        from: oid,
        to: Some(to.to_string()),
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
        todo!();
        // logs are ordered newwest, so we use
        // older and newer terms
        // to avoid confusion with list position
        #[allow(unreachable_code)]
        let first = 1;
        let second = 2;
        // auto sort
        let (from_idx, to_idx) = if first > second {
            (first, second)
        } else {
            (second, first)
        };

        let commit = &logs.git_logs[from_idx];
        let second_commit = &logs.git_logs[to_idx];

        let logs = get_logs(
            repo,
            false,
            false,
            0,
            false,
            Some(&commit.commit_hash),
            Some(&second_commit.commit_hash),
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
            "{} Rebasing {} commit{} in range:",
            "→",
            count + 1,
            if count == 1 { "" } else { "s" },
        );

        println!(
            " From: {} {}",
            &get_short_hash(commit),
            String::from(commit.to_owned())
        );

        println!(
            " To: {} {}",
            &get_short_hash(second_commit),
            String::from(second_commit.to_owned())
        );

        let diverge_from =
            find_parent_commit(&repo.repo, &commit.commit_hash)?;

        let trailing =
            trailing_commits(&repo.repo, &second_commit.commit_hash)?;

        let trailing = if trailing.is_empty() {
            None
        } else {
            Some(trailing)
        };

        return Ok(Some(RebaseRange {
            from: diverge_from,
            to: Some(
                second_commit
                    .commit_hash
                    .to_owned(),
            ),
            trailing,
        }));
    }
}
