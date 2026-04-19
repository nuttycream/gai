use git2::Oid;

use crate::{
    git::{
        GitRepo,
        branch::{find_divergence_branch, get_diverged_branches},
    },
    print::input,
};

pub(super) fn rebase_branch(
    git: &GitRepo,
    div_branch_arg: Option<&str>,
    interactive: bool,
) -> anyhow::Result<Option<Oid>> {
    if interactive {
        return divergence_flow(git);
    }

    if let Some(div_branch_arg) = div_branch_arg {
        let oid = find_divergence_branch(&git.repo, div_branch_arg)?;

        println!(
            "{} Using divergence from branch: {}",
            "→", div_branch_arg
        );

        Ok(Some(oid))
    } else {
        println!("No arg");
        Ok(None)
    }
}

fn divergence_flow(repo: &GitRepo) -> anyhow::Result<Option<Oid>> {
    let branches = get_diverged_branches(&repo.repo)?;

    let typed_branch = input::prompt("Specify branch")?;

    let branch = if let Some(b) = branches
        .iter()
        .find(|b| b.name == typed_branch)
    {
        b
    } else {
        todo!()
    };

    let commit_oid = if let Some(d) = branch
        .divergence
        .to_owned()
    {
        d.merge_base
    } else {
        println!(
            "No merge_base available... exiting, this shouldn't happen"
        );
        return Ok(None);
    };

    Ok(Some(commit_oid))
}
