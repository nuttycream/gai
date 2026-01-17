use anyhow::Context;
use git2::{Branch, BranchType, Oid, Repository};

use super::errors::GitError;

/// finds the divergence
/// commit from a specified
/// spec str
pub fn find_divergence_branch(
    repo: &Repository,
    spec: &str,
) -> anyhow::Result<Oid> {
    let head_oid = repo
        .head()?
        .target()
        .ok_or(GitError::NoHead)
        .with_context(|| "HEAD has no target, detached")?;

    let divergent_oid = repo
        .revparse_single(spec)?
        .id();

    let base = repo.merge_base(head_oid, divergent_oid)?;

    if !repo.graph_descendant_of(head_oid, base)? {
        return Err(GitError::Generic(format!(
            "{} is not ancestor of HEAD",
            spec
        ))
        .into());
    }

    Ok(base)
}

/// realistically dont need this,
/// since root would fail
pub fn validate_branch_exists(
    repo: &Repository,
    name: &str,
) -> anyhow::Result<bool> {
    let valid = Branch::name_is_valid(name)?;

    let exists = repo
        .find_branch(name, BranchType::Local)
        .is_ok();

    Ok(valid && exists)
}
