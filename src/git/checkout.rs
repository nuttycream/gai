use git2::Repository;

use super::{
    errors::GitError, status::is_workdir_clean, utils::bytes2string,
};

/// Detach HEAD to point to a commit then checkout HEAD,
/// does not work if there are uncommitted changes
/// takes in a commit hash str, not a raw Oid
pub fn checkout_commit(
    repo: &Repository,
    commit_hash: &str,
) -> anyhow::Result<()> {
    let cur_ref = repo.head()?;

    let statuses = repo.statuses(Some(
        git2::StatusOptions::new().include_ignored(false),
    ))?;

    let oid = git2::Oid::from_str(commit_hash)?;

    if statuses.is_empty() {
        repo.set_head_detached(oid)?;

        if let Err(e) = repo.checkout_head(Some(
            git2::build::CheckoutBuilder::new().force(),
        )) {
            repo.set_head(
                bytes2string(cur_ref.name_bytes())?.as_str(),
            )?;
            return Err(GitError::Git2(e).into());
        }
        Ok(())
    } else {
        Err(GitError::Generic("Uncommited Changes".to_string())
            .into())
    }
}

/// sync the curr working tree and index,
/// with additional safety
/// so that we don't lose commits
/// errors on an unclean workdir
pub fn force_checkout_head(repo: &Repository) -> anyhow::Result<()> {
    if !is_workdir_clean(repo)? {
        return Err(anyhow::anyhow!("working tree is not clean",));
    }

    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::new().force(),
    ))?;

    Ok(())
}
