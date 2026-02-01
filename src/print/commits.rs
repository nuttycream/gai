use console::{Color, Style, style};
use dialoguer::{Select, theme::ColorfulTheme};

use crate::schema::commit::{CommitSchema, PrefixType};

use super::tree::{Tree, TreeItem};

pub fn get_prefix_color(prefix: &PrefixType) -> Color {
    match prefix {
        PrefixType::Feat => Color::Green,
        PrefixType::Fix => Color::Red,
        //urange
        PrefixType::Refactor => Color::Color256(214),
        _ => Color::White,
    }
}

/// display the responsecommits
/// before converting to usable
/// git commits
/// returns an selected option
pub fn print_response_commits(
    commits: &[CommitSchema],
    compact: bool,
    as_hunks: bool,
    skip_confirmation: bool,
) -> anyhow::Result<Option<usize>> {
    let mut items = Vec::new();

    for (i, commit) in commits
        .iter()
        .enumerate()
    {
        let mut commit_children = Vec::new();

        // if we need to we might have to truncate this
        // similar to the body, but i foresee this as a non-issue?
        if !commit
            .header
            .is_empty()
        {
            let header_item = TreeItem::new_leaf(
                format!("commit_{}_header", i),
                &commit.header,
            )
            .style(Style::new().white());

            commit_children.push(header_item);
        }

        // preview the body if exists
        if let Some(ref body) = commit.body {
            let truncated_body = if body.len() > 20 {
                format!("{}...", &body[..20])
            } else {
                body.to_owned()
            };

            let body_item = TreeItem::new_leaf(
                "body".to_owned(),
                &truncated_body,
            )
            .style(Style::new().dim());

            commit_children.push(body_item);
        }

        let mut files = Vec::new();

        // for single path, push it, otherwise use all paths
        let mut paths = Vec::new();

        if let Some(ref p) = commit.path {
            paths.push(p);
        }

        if let Some(ref ps) = commit.paths {
            paths.extend(ps.iter());
        }

        for file in paths {
            let file_display = format!("{}", style(file).magenta());

            // add the hunks as one-line
            // branch
            // not shown if staging as hunks
            // is not selected
            if as_hunks {
                let mut hunk_idxes = Vec::new();

                // todo this is a little out of scope
                // imo, this should be handled
                // within ResponseCommits, for
                // hunk assignment
                let hunk_ids = commit
                    .hunk_ids
                    .as_deref()
                    .unwrap_or(&[]);
                for hunk_id in hunk_ids {
                    if let Some((path, index)) =
                        hunk_id.split_once(':')
                        && path == file
                    {
                        hunk_idxes.push(index.to_owned());
                    }
                }

                let mut file_children = Vec::new();

                let hunks_display = format!(
                    "Hunks: [{}]",
                    style(hunk_idxes.join(", ")).magenta()
                );

                let hunks_item = TreeItem::new_leaf(
                    format!("{}_hunks", file),
                    &hunks_display,
                )
                .style(Style::new().dim());

                file_children.push(hunks_item);

                let file_item = TreeItem::new(
                    file.clone(),
                    file_display,
                    file_children,
                )?
                .style(Style::new().dim());

                //commit_children.push(file_item);
                files.push(file_item);
            } else {
                let file_item =
                    TreeItem::new_leaf(file.clone(), &file_display)
                        .style(Style::new().dim());

                //commit_children.push(file_item);
                files.push(file_item);
            }
        }

        let files_display = format!(
            "{}",
            style(format!("Files ({})", files.len())).dim()
        );

        let files_item = TreeItem::new(
            format!("commit_{}_files", i),
            files_display,
            files,
        )?
        .style(Style::new().dim());

        commit_children.push(files_item);

        // build prefix(scope)
        // ignoring commit message
        // processing, since this would
        // trigger afterwards
        // when converting CommitSchemas -> GitCommits
        let prefix = match &commit.scope {
            Some(s) if !s.is_empty() => format!(
                "{}({})",
                commit
                    .prefix
                    .to_string()
                    .to_lowercase(),
                s
            ),
            _ => commit
                .prefix
                .to_string()
                .to_lowercase(),
        };

        let color = get_prefix_color(&commit.prefix);

        let commit_idx = style(format!("[{}]", i)).dim();

        let display = if compact {
            let prefix =
                style(format!("{}: {}", prefix, commit.header))
                    .fg(color)
                    .bold();

            format!("{} {}", commit_idx, prefix)
        } else {
            let prefix = style(format!("{}:", prefix))
                .fg(color)
                .bold();
            format!("{} {}", commit_idx, prefix)
        };

        // when we implement
        // fuzzy selection to trim
        // commits we dont want
        // or regenerate
        let for_fuzzy_id = format!("{}: {}", prefix, commit.header);

        let item =
            TreeItem::new(for_fuzzy_id, display, commit_children)?
                .style(
                    Style::new()
                        .fg(color)
                        .bold(),
                );

        items.push(item);
    }

    if !items.is_empty() {
        Tree::new(&items)?
            .collapsed(compact)
            .style(
                Style::new()
                    .dim()
                    .dim(),
            )
            .render();
    }

    if skip_confirmation {
        return Ok(None);
    }

    let options = ["Apply", "Regenerate", "Exit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option:")
        .items(options)
        .default(0)
        .interact_opt()?;

    Ok(selection)
}
