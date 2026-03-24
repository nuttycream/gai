use crossterm::style::{Attribute, Color, ContentStyle, Stylize};

use crate::schema::commit::{CommitSchema, PrefixType};

use super::{
    renderer::Renderer,
    tree::{Tree, TreeItem},
};

/// display the responsecommits
/// before converting to usable
/// git commits
/// returns an selected option
pub fn response_commits(
    renderer: &Renderer,
    commits: &[CommitSchema],
    as_hunks: bool,
) -> anyhow::Result<()> {
    let mut items = Vec::new();
    let max_length = renderer
        .width
        .saturating_sub(3) as usize;

    let dim = ContentStyle::new().attribute(Attribute::Dim);
    let white = ContentStyle::new().with(Color::White);

    let magenta_dim = ContentStyle::new()
        .with(Color::Magenta)
        .attribute(Attribute::Dim);

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
            .style(white);

            commit_children.push(header_item);
        }

        // preview the body if exists
        if let Some(ref body) = commit.body {
            let body = format!("{}...", &body[..max_length]);

            let body_item =
                TreeItem::new_leaf("body".to_owned(), &body)
                    .style(dim);

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
            let file_display = format!("{}", file);

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

                let hunks_display =
                    format!("Hunks: [{}]", hunk_idxes.join(", "));

                let hunks_item = TreeItem::new_leaf(
                    format!("{}_hunks", file),
                    &hunks_display,
                )
                .style(dim);

                file_children.push(hunks_item);

                let file_item = TreeItem::new(
                    file.clone(),
                    file_display,
                    file_children,
                )?
                .style(magenta_dim);

                //commit_children.push(file_item);
                files.push(file_item);
            } else {
                let file_item =
                    TreeItem::new_leaf(file.clone(), &file_display)
                        .style(magenta_dim);

                //commit_children.push(file_item);
                files.push(file_item);
            }
        }

        let files_display =
            format!("{}", format!("Files ({})", files.len()));

        let files_item = TreeItem::new(
            format!("commit_{}_files", i),
            files_display,
            files,
        )?
        .style(dim);

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

        let color = prefix_color(&commit.prefix);

        let colored = if renderer
            .style
            .allow_bold
        {
            ContentStyle::new()
                .with(color)
                .attribute(Attribute::Bold)
        } else {
            ContentStyle::new().with(color)
        };

        let commit_idx = format!("[{}]", i);

        let display = if renderer.compact {
            let prefix = format!("{}: {}", prefix, commit.header);

            format!("{} {}", commit_idx, prefix)
        } else {
            let prefix = format!("{}:", prefix);

            format!("{} {}", commit_idx, prefix)
        };

        // when we implement
        // fuzzy selection to trim
        // commits we dont want
        // or regenerate
        let for_fuzzy_id = format!("{}: {}", prefix, commit.header);

        let item =
            TreeItem::new(for_fuzzy_id, display, commit_children)?
                .style(colored);

        items.push(item);
    }

    if !items.is_empty() {
        Tree::new(&items)?
            .collapsed(renderer.compact)
            .style(dim)
            .render();
    }

    Ok(())
}

fn prefix_color(prefix: &PrefixType) -> Color {
    match prefix {
        PrefixType::Feat => Color::Green,
        PrefixType::Fix => Color::Red,
        //urange
        PrefixType::Refactor => Color::Rgb {
            r: 255,
            g: 127,
            b: 80,
        },
        _ => Color::White,
    }
}
