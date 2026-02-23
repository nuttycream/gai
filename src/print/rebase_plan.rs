use console::{Style, style};

use crate::schema::rebase_plan::{
    PlanOperationKind, PlanOperationSchema,
};

use super::tree::TreeItem;

/// display rebase_plan
/// in a tree format
pub fn print_rebase_plan(
    ops: &[PlanOperationSchema],
    compact: bool,
) -> anyhow::Result<()> {
    println!(
        "Generated Rebase Plan with {} Operation{}",
        style(ops.len()).bold(),
        if ops.len() == 1 { "" } else { "s" }
    );

    let mut items = Vec::new();

    for (i, op) in ops
        .iter()
        .enumerate()
    {
        let mut children = Vec::new();

        let reason_item = TreeItem::new_leaf(
            format!("reason_{}", i),
            format!(
                "{} {}",
                style("Reasoning:").dim(),
                style(&op.reasoning).italic()
            ),
        )
        .style(Style::new().dim());
        children.push(reason_item);

        if let Some(ref msg) = op.new_message {
            let truncated = if msg.len() > 72 {
                format!("{}...", &msg[..72])
            } else {
                msg.clone()
            };

            let msg_item = TreeItem::new_leaf(
                format!("msg_{}", i),
                format!(
                    "{} {}",
                    style("New Message:").dim(),
                    style(&truncated).italic()
                ),
            )
            .style(Style::new().dim());
            children.push(msg_item);
        }

        if let Some(squash_target) = op.squash_with {
            let squash_item = TreeItem::new_leaf(
                format!("squash_{}", i),
                format!(
                    "{} {}",
                    style("Squash With:").dim(),
                    style(format!("Commit [{}]", squash_target))
                        .cyan()
                        .bold()
                ),
            )
            .style(Style::new().dim());
            children.push(squash_item);
        }

        let op_style = match op.operation {
            PlanOperationKind::Pick => Style::new().green(),
            PlanOperationKind::Reword => Style::new().yellow(),
            PlanOperationKind::Squash => Style::new().magenta(),
            PlanOperationKind::Drop => Style::new().red(),
        };

        let op_idx = style(format!("[{}]", op.commit_id)).dim();
        let op_label = op_style.apply_to(
            op.operation
                .to_owned(),
        );

        let display = if !compact {
            let preview = match (&op.operation, &op.new_message) {
                (PlanOperationKind::Squash, _) => op
                    .squash_with
                    .map(|s| format!("→ commit [{}]", s))
                    .unwrap_or_default(),
                (_, Some(msg)) => {
                    if msg.len() > 50 {
                        format!("{}...", &msg[..50])
                    } else {
                        msg.clone()
                    }
                }
                _ => String::new(),
            };

            format!(
                "{} {} {}",
                op_idx,
                op_label,
                style(preview).dim()
            )
        } else {
            format!("{} {}", op_idx, op_label)
        };

        let item =
            TreeItem::new(format!("op_{}", i), display, children)?
                .style(op_style);

        items.push(item);
    }

    Ok(())
}
