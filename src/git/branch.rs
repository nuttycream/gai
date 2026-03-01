use anyhow::Context;
use git2::{BranchType, Oid, Repository};
use std::collections::HashSet;

use super::{errors::GitError, utils::bytes2string};

#[derive(Clone, Debug)]
pub struct LocalBranch {
    pub is_head: bool,
    pub has_upstream: bool,
    pub upstream: Option<UpstreamBranch>,
    pub remote: Option<String>,
}

#[derive(Clone, Debug)]
pub struct UpstreamBranch {
    pub reference: String,
}

#[derive(Clone, Debug)]
pub struct RemoteBranch {
    pub has_tracking: bool,
}

#[derive(Clone, Debug)]
pub enum BranchDetails {
    Local(LocalBranch),
    Remote(RemoteBranch),
}

/// branch info from asyncgit
#[derive(Clone, Debug)]
pub struct BranchInfo {
    pub name: String,

    /// full ref path, i.e refs/head/main
    pub reference: String,

    pub top_commit_message: String,

    pub details: BranchDetails,

    pub divergence: Option<BranchDivergence>,
}

/// formerly BranchCompare
/// used to find the most recent
/// ancestor
#[derive(Clone, Debug)]
pub struct BranchDivergence {
    pub merge_base: Oid,
    pub ahead: usize,
    pub behind: usize,
}

impl BranchInfo {
    /// returns details about local branch or None
    pub const fn local_details(&self) -> Option<&LocalBranch> {
        if let BranchDetails::Local(details) = &self.details {
            return Some(details);
        }

        None
    }

    /// returns whether branch is local
    pub const fn is_local(&self) -> bool {
        matches!(self.details, BranchDetails::Local(_))
    }
}

/// finds the divergence
/// commit from a specified
/// spec str
pub fn find_divergence_branch(
    repo: &Repository,
    spec: &str,
) -> anyhow::Result<Oid> {
    let head_oid = get_head_oid(repo)?;

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

/// collect all branches that the
/// current branch diverged from
pub fn get_diverged_branches(
    repo: &Repository
) -> anyhow::Result<Vec<BranchInfo>> {
    let branches = get_branches_info(repo, true)?
        .into_iter()
        .filter(|b| {
            b.divergence
                .as_ref()
                .map(|d| d.ahead > 0)
                .unwrap_or(false)
        })
        .collect();

    Ok(branches)
}

/// finds a single branch that matches
/// spec, returns None if no matching branch
/// name or the branch is diverging branch
/// is not ahead
pub fn find_diverged_branch(
    repo: &Repository,
    branch: &str,
) -> anyhow::Result<Option<BranchInfo>> {
    let branches = get_branches_info(repo, true)?;

    let diverged = branches
        .into_iter()
        .find(|b| {
            b.name == branch
                && b.divergence
                    .as_ref()
                    .map(|d| d.ahead > 0)
                    .unwrap_or(false)
        });

    Ok(diverged)
}

/// returns a list of `BranchInfo` with a
/// simple summary on each branch
/// `local` filters for local branches otherwise
/// remote branches will be returned
///
/// modified to include divergence point
fn get_branches_info(
    repo: &Repository,
    local: bool,
) -> anyhow::Result<Vec<BranchInfo>> {
    let (filter, remotes_with_tracking) = if local {
        (BranchType::Local, HashSet::default())
    } else {
        let remotes: HashSet<_> = repo
            .branches(Some(BranchType::Local))?
            .filter_map(|b| {
                let branch = b.ok()?.0;
                let upstream = branch.upstream();
                upstream
                    .ok()?
                    .name_bytes()
                    .ok()
                    .map(ToOwned::to_owned)
            })
            .collect();
        (BranchType::Remote, remotes)
    };

    let mut branches_for_display: Vec<BranchInfo> = repo
        .branches(Some(filter))?
        .map(|b| {
            let branch = b?.0;

            let top_commit = branch
                .get()
                .peel_to_commit()?;

            let reference = bytes2string(
                branch
                    .get()
                    .name_bytes(),
            )?;

            let upstream = branch.upstream();

            let remote = repo
                .branch_upstream_remote(&reference)
                .ok()
                .as_ref()
                .and_then(git2::Buf::as_str)
                .map(String::from);

            let name_bytes = branch.name_bytes()?;

            let upstream_branch = upstream
                .ok()
                .and_then(|upstream| {
                    bytes2string(
                        upstream
                            .get()
                            .name_bytes(),
                    )
                    .ok()
                    .map(|reference| UpstreamBranch { reference })
                });

            let details = if local {
                BranchDetails::Local(LocalBranch {
                    is_head: branch.is_head(),
                    has_upstream: upstream_branch.is_some(),
                    upstream: upstream_branch,
                    remote,
                })
            } else {
                BranchDetails::Remote(RemoteBranch {
                    has_tracking: remotes_with_tracking
                        .contains(name_bytes),
                })
            };

            let head_oid = get_head_oid(repo)?;

            let divergence = {
                let branch_oid = branch
                    .get()
                    .peel_to_commit()?
                    .id();

                // skp if this is HEAD
                if branch.is_head() {
                    None
                } else if let Ok(merge_base) =
                    repo.merge_base(head_oid, branch_oid)
                {
                    let (ahead, behind) = repo
                        .graph_ahead_behind(head_oid, branch_oid)?;

                    Some(BranchDivergence {
                        merge_base,
                        ahead,
                        behind,
                    })
                } else {
                    // no common ancestor
                    // caller checks divergence.is_none()
                    None
                }
            };

            Ok(BranchInfo {
                name: bytes2string(name_bytes)?,
                reference,
                top_commit_message: bytes2string(
                    top_commit
                        .summary_bytes()
                        .unwrap_or_default(),
                )?,
                details,
                divergence,
            })
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    branches_for_display.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(branches_for_display)
}

/// returns the head of the current branch
pub(super) fn get_head_oid(repo: &Repository) -> anyhow::Result<Oid> {
    repo.head()?
        .target()
        .ok_or(GitError::NoHead)
        .with_context(|| "HEAD has no target, detached")
}
