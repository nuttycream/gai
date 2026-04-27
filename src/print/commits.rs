use std::io::Write;

use anstream::stdout;
use owo_colors::OwoColorize;

use crate::{print::utils::tput_size, schema::commit::CommitSchema};

use super::tree::{Tree, TreeItem};

/// display the responsecommits
/// before converting to usable
/// git commits
/// returns an selected option
pub fn response_commits(
    commits: &[CommitSchema],
    as_hunks: bool,
) -> anyhow::Result<()> {
    let mut items = Vec::new();

    let (width, _) = tput_size().unwrap_or((80, 100));
    let max_length = width.saturating_sub(5) as usize;

    for (i, commit) in commits
        .iter()
        .enumerate()
    {
        let mut commit_children = Vec::new();

        // preview the body if exists
        if let Some(ref body) = commit.body {
            let truncated = body
                .chars()
                .take(max_length)
                .collect::<String>();

            let body = if truncated.len() < body.len() {
                format!("{truncated}...")
            } else {
                truncated
            };

            let body_item =
                TreeItem::new_leaf("body".to_owned(), &body);

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
            let file_display = file.to_string();

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
                );

                file_children.push(hunks_item);

                let file_item = TreeItem::new(
                    file.clone(),
                    file_display,
                    file_children,
                )?;

                //commit_children.push(file_item);
                files.push(file_item);
            } else {
                let file_item =
                    TreeItem::new_leaf(file.clone(), &file_display);

                //commit_children.push(file_item);
                files.push(file_item);
            }
        }

        let files_display = format!("Files ({})", files.len());

        let files_item = TreeItem::new(
            format!("commit_{}_files", i),
            files_display,
            files,
        )?;

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

        let commit_idx = format!("[{}]", i + 1);

        let display =
            format!("{} {}: {}", commit_idx, prefix, commit.header);

        let display = display
            .style(
                commit
                    .prefix
                    .style(),
            )
            .to_string();

        // when we implement
        // fuzzy selection to trim
        // commits we dont want
        // or regenerate
        let for_fuzzy_id = format!("{}: {}", prefix, commit.header);

        let item =
            TreeItem::new(for_fuzzy_id, display, commit_children)?;

        items.push(item);
    }

    if !items.is_empty() {
        Tree::new(&items)?.render(&mut stdout());
    }

    Ok(())
}

pub(crate) fn completed_commit(
    branch_name: &str,
    hash: &str,
    commit_msg: &str,
    files_changed: usize,
    insertions: usize,
    deletions: usize,
) -> anyhow::Result<()> {
    let mut out = stdout();

    let short = &hash[..7];
    let file = if files_changed == 1 { "file" } else { "files" };
    let inserts = if insertions == 1 {
        "insertion(+)"
    } else {
        "insertions(+)"
    };
    let delets = if deletions == 1 {
        "deletion(-)"
    } else {
        "deletions(-)"
    };

    write!(
        out,
        "\n[{} {}] {}\n {} {} changed, {} {}, {} {}\n",
        branch_name,
        short,
        commit_msg,
        files_changed,
        file,
        insertions,
        inserts,
        deletions,
        delets,
    )?;

    Ok(())
}
