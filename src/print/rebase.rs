use console::{Style, style};

use crate::git::branch::{BranchDetails, BranchInfo};

use super::tree::{Tree, TreeItem};

/// display branch_info in a tree format
pub fn print_branches_info(
    branches: &[BranchInfo],
    compact: bool,
) -> anyhow::Result<Vec<&str>> {
    let mut items = Vec::new();

    for (i, branch) in branches
        .iter()
        .enumerate()
    {
        let mut branch_children = Vec::new();

        // reference path
        /* let ref_item = TreeItem::new_leaf(
            format!("branch_{}_ref", i),
            format!("{}", style(&branch.reference).dim()),
        )
        .style(Style::new().dim());
        branch_children.push(ref_item); */

        // top commit message but truncated
        if !branch
            .top_commit_message
            .is_empty()
        {
            let truncated_msg = if branch
                .top_commit_message
                .len()
                > 50
            {
                format!("{}...", &branch.top_commit_message[..50])
            } else {
                branch
                    .top_commit_message
                    .clone()
            };

            let commit_item = TreeItem::new_leaf(
                format!("branch_{}_commit", i),
                format!(
                    "{} {}",
                    style("Commit Message:").dim(),
                    style(&truncated_msg).italic()
                ),
            )
            .style(Style::new().dim());
            branch_children.push(commit_item);
        }

        // branch details
        match &branch.details {
            BranchDetails::Local(local) => {
                if let Some(ref upstream) = local.upstream {
                    let upstream_item = TreeItem::new_leaf(
                        format!("branch_{}_upstream", i),
                        format!(
                            "{} {}",
                            style("Upstream:").dim(),
                            style(&upstream.reference).cyan()
                        ),
                    )
                    .style(Style::new().dim());
                    branch_children.push(upstream_item);
                }
            }
            BranchDetails::Remote(remote) => {
                if remote.has_tracking {
                    let tracking_item = TreeItem::new_leaf(
                        format!("branch_{}_tracking", i),
                        format!(
                            "{}",
                            style("has local tracking branch").dim()
                        ),
                    )
                    .style(Style::new().dim());
                    branch_children.push(tracking_item);
                }
            }
        }

        // divergence info
        if let Some(ref div) = branch.divergence {
            let ahead_behind = format!(
                "{} {} {} {}",
                style("↑").green(),
                style(div.ahead)
                    .green()
                    .bold(),
                style("↓").red(),
                style(div.behind)
                    .red()
                    .bold()
            );

            let div_item = TreeItem::new_leaf(
                format!("branch_{}_divergence", i),
                ahead_behind,
            )
            .style(Style::new().dim());
            branch_children.push(div_item);
        }

        // build the branch display name
        let branch_color = if branch.is_local() {
            Style::new().yellow()
        } else {
            Style::new().red()
        };

        let branch_idx = style(format!("[{}]", i)).dim();

        let display = if compact {
            // for compact just show name and divergence if it has Some
            let divergence_str = branch
                .divergence
                .as_ref()
                .map(|d| format!(" [↑{} ↓{}]", d.ahead, d.behind))
                .unwrap_or_default();

            format!(
                "{} {}{}",
                branch_idx,
                branch_color.apply_to(&branch.name),
                style(divergence_str).dim()
            )
        } else {
            format!(
                "{} {}",
                branch_idx,
                branch_color.apply_to(&branch.name)
            )
        };

        let item = TreeItem::new(
            branch.name.clone(),
            display,
            branch_children,
        )?
        .style(branch_color);

        items.push(item);
    }

    Tree::new(&items)?
        .collapsed(compact)
        .style(Style::new().dim())
        .render();

    Ok(branches
        .iter()
        .map(|b| b.name.as_str())
        .collect())
}
