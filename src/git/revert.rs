use git2::{Oid, Repository};

/// https://github.com/gitui-org/gitui/blob/master/asyncgit/src/sync/commit_revert.rs
/// revert
pub fn revert_commit(
    repo: &Repository,
    commit: &str,
) -> anyhow::Result<()> {
    let oid = Oid::from_str(commit)?;
    let commit = repo.find_commit(oid)?;

    repo.revert(&commit, None)?;

    Ok(())
}
