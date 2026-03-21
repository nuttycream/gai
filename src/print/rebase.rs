use crate::git::branch::{BranchDetails, BranchInfo};

use super::tree::{Tree, TreeItem};

/// display branch_info in a tree format
pub fn print_branches_info(
    branches: &[BranchInfo],
    compact: bool,
) -> anyhow::Result<Vec<&str>> {
    Ok(Vec::new())
}
