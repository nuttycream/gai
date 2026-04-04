use git2::Oid;

use crate::{
    git::{GitRepo, commit::find_parent_commit, log::get_logs},
    print::input_prompt,
};

pub(super) fn rebase_last(
    git: &GitRepo,
    is_interactive: bool,
    last_n: Option<usize>,
) -> anyhow::Result<Option<Oid>> {
    if is_interactive {
        return last_n_flow(git);
    }

    if let Some(last_n) = last_n {
        let logs = get_logs(
            git, false, false, last_n, false, None, None, None,
        )?;

        if last_n > logs.git_logs.len() {
            println!(
                "Warning: Only {} commits exist in history but you requested {}",
                logs.git_logs.len(),
                last_n
            );
        }

        // this should get the last logged commit
        // if the count exceeds, get_logs()
        // will handle that and return or "take"
        // the last commit
        let oldest_commit_hash = logs
            .git_logs
            .last()
            .map(|l| {
                l.commit_hash
                    .to_owned()
            })
            .unwrap();

        let oid = find_parent_commit(&git.repo, &oldest_commit_hash)?;

        println!(
            "{} Rebasing last {} commit{}",
            "→",
            last_n,
            if last_n == 1 { "" } else { "s" }
        );

        Ok(Some(oid))
    } else {
        Ok(None)
    }
}

fn last_n_flow(repo: &GitRepo) -> anyhow::Result<Option<Oid>> {
    let n: usize;

    loop {
        let input =
            match input_prompt("Specify a valid number", None)? {
                Some(i) => i,
                None => {
                    println!("Exiting...");
                    return Ok(None);
                }
            };

        match input.parse::<usize>() {
            Ok(v) => {
                if v == 0 {
                    println!("Please enter a value greater than 0");
                    continue;
                }

                n = v;
                break;
            }
            Err(_) => {
                println!("Cannot parse {} as a valid number", input);
                continue;
            }
        }
    }

    let logs =
        get_logs(repo, false, false, n, false, None, None, None)?;

    // if n exceeds log length, continue, regardless
    if n > logs.git_logs.len() {
        println!(
            "Only {} commits exist in history but you requested {}",
            logs.git_logs.len(),
            n
        );
    }

    // this should get the last logged commit
    // if the count exceeds, get_logs()
    // will handle that and return or "take"
    // the last commit
    let oldest_commit_hash = match logs
        .git_logs
        .last()
        .map(|l| {
            l.commit_hash
                .to_owned()
        }) {
        Some(h) => h,
        None => {
            println!("No Commits Found, Exiting...");
            return Ok(None);
        }
    };

    let oid = find_parent_commit(&repo.repo, &oldest_commit_hash)?;

    Ok(Some(oid))
}
