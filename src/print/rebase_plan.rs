use crate::schema::rebase_plan::{
    PlanOperationKind, PlanOperationSchema,
};

use super::{
    option_prompt,
    tree::{Tree, TreeItem},
};

/// display rebase_plan
/// in a tree format
pub fn print_rebase_plan(
    ops: &[PlanOperationSchema],
    compact: bool,
) -> anyhow::Result<Option<usize>> {
    Ok(Some(0))
}
