use console::style;
use git2::Oid;

use crate::{
    git::{
        GitRepo,
        branch::{find_divergence_branch, get_diverged_branches},
    },
    print::{print_choice_prompt, rebase::print_branches_info},
};

pub(super) fn rebase_branch(
    git: &GitRepo,
    div_branch_arg: Option<&str>,
    interactive: bool,
    compact: bool,
) -> anyhow::Result<Option<Oid>> {
    if interactive {
        return divergence_flow(git, compact);
    }

    if let Some(div_branch_arg) = div_branch_arg {
        let oid = find_divergence_branch(&git.repo, div_branch_arg)?;

        println!(
            "{} Using divergence from branch: {}",
            style("→").green(),
            style(div_branch_arg).cyan()
        );

        Ok(Some(oid))
    } else {
        println!("No arg");
        Ok(None)
    }
}

fn divergence_flow(
    repo: &GitRepo,
    compact: bool,
) -> anyhow::Result<Option<Oid>> {
    let branches = get_diverged_branches(&repo.repo)?;

    let opts = print_branches_info(&branches, compact)?;

    let selected_branch = if let Some(b) =
        print_choice_prompt(&opts, None, Some("Select a Branch"))?
    {
        b
    } else {
        println!("Exiting...");
        return Ok(None);
    };

    let commit_oid = if let Some(d) = branches[selected_branch]
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
