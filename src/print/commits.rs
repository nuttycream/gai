use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{
        Attribute, Color, ContentStyle, Print, ResetColor, SetStyle,
        Stylize,
    },
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};
use std::io::{Write, stdout};

use crate::{
    git::{Diffs, diffs::DiffLineType},
    schema::commit::{CommitSchema, PrefixType},
};

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

    let allow_colors = renderer
        .style
        .allow_colors;

    let width = terminal::size()?.1;
    let max_length = width.saturating_sub(3) as usize;

    // avoiding the rewriting of treeitem, since it takes
    // in a style
    let no = ContentStyle::new();

    let dim = if allow_colors {
        ContentStyle::new().attribute(Attribute::Dim)
    } else {
        no
    };

    let white = if allow_colors {
        ContentStyle::new().with(Color::White)
    } else {
        no
    };

    let magenta_dim = if allow_colors {
        ContentStyle::new()
            .with(Color::Magenta)
            .attribute(Attribute::Dim)
    } else {
        no
    };

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

        let files_display = format!("Files ({})", files.len());

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

        let colored = if allow_colors {
            let color = prefix_color(&commit.prefix);
            ContentStyle::new().with(color)
        } else {
            no
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

/// full screen scrollable view of the full response
/// using alt screen
/// has a builtin event handler, and can exit out
/// ideally, u should handle this via reusable menu
pub fn full_response(
    renderer: &Renderer,
    commits: &[CommitSchema],
    diffs: &Diffs,
) -> anyhow::Result<()> {
    let allow_colors = renderer
        .style
        .allow_colors;

    let (width, height) = terminal::size()?;

    let lines = response_lines(commits, diffs, allow_colors, width);

    event_handler(&lines, height)
}

fn event_handler(
    lines: &[String],
    height: u16,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, cursor::Hide)?;
    let mut offset = 0;

    loop {
        let visible = height.saturating_sub(1) as usize;

        execute!(out, terminal::Clear(terminal::ClearType::All))?;

        for (i, line) in lines
            .iter()
            .skip(offset)
            .take(visible)
            .enumerate()
        {
            execute!(out, cursor::MoveTo(0, i as u16), Print(line),)?;
        }

        let status = format!(
            "Lines {}-{} of {} | j/k scroll | {{ jump up | }} jump down | g top | G bottom | q exit ",
            offset + 1,
            (offset + visible).min(lines.len()),
            lines.len()
        );

        execute!(
            out,
            cursor::MoveTo(0, height.saturating_sub(1)),
            SetStyle(
                ContentStyle::new().attribute(Attribute::Reverse)
            ),
            Print(&status),
            ResetColor,
        )?;

        out.flush()?;

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c')
                    if modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    break;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    offset = offset.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if offset + visible < lines.len() {
                        offset = offset.saturating_add(1);
                    }
                }
                KeyCode::Char('{') => {
                    offset = offset.saturating_sub(25);
                }
                KeyCode::Char('}') => {
                    if offset + visible < lines.len() {
                        offset = offset.saturating_add(25);
                    }
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    offset = 0;
                }
                KeyCode::End | KeyCode::Char('G') => {
                    offset = lines
                        .len()
                        .saturating_sub(visible);
                }
                _ => {}
            }
        }
    }

    execute!(out, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

/// build all the lines for the full response view
/// each string in the list should already contain crossterm
/// colors
fn response_lines(
    commits: &[CommitSchema],
    diffs: &Diffs,
    allow_colors: bool,
    width: u16,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    for (i, commit) in commits
        .iter()
        .enumerate()
    {
        let color = if allow_colors {
            prefix_color(&commit.prefix)
        } else {
            Color::Reset
        };

        let prefix_str = match &commit.scope {
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

        // separator
        if allow_colors {
            lines.push(format!(
                "{}",
                "-".repeat(width as usize)
                    .dim()
            ));
        } else {
            lines.push("-".repeat(width as usize));
        }

        // header desc
        if allow_colors {
            lines.push(format!(
                "{} {}",
                format!("[{}] {}:", i, prefix_str)
                    .with(color)
                    .bold(),
                commit
                    .header
                    .clone()
                    .white(),
            ));
        } else {
            lines.push(format!(
                "[{}] {}: {}",
                i, prefix_str, commit.header
            ));
        }

        // body
        if let Some(ref body) = commit.body {
            lines.push(String::new());
            for body_line in body.lines() {
                lines.push(body_line.to_string());
            }
        }

        // reasoning
        for line in commit
            .reasoning
            .lines()
        {
            if allow_colors {
                lines.push(format!("{}", line.dim()));
            } else {
                lines.push(line.to_string());
            }
        }

        lines.push(String::new());
        // files + diffs
        let mut paths: Vec<&String> = Vec::new();
        if let Some(ref p) = commit.path {
            paths.push(p);
        }
        if let Some(ref ps) = commit.paths {
            paths.extend(ps.iter());
        }

        if !paths.is_empty() {
            lines.push(String::new());
            if allow_colors {
                lines.push(format!(
                    "{}",
                    format!("Files ({}):", paths.len()).dim()
                ));
            } else {
                lines.push(format!("Files ({}):", paths.len()));
            }
        }

        for path in &paths {
            if allow_colors {
                lines.push(format!(
                    "{}",
                    path.as_str()
                        .magenta()
                        .dim()
                ));
            } else {
                lines.push(path.to_string());
            }

            // gotta find the matching filediff then
            // render its hunks
            if let Some(file_diff) = diffs
                .files
                .iter()
                .find(|f| &f.path == *path)
            {
                let hunk_ids: Option<Vec<usize>> = commit
                    .hunk_ids
                    .as_ref()
                    .and_then(|ids| {
                        let filtered: Vec<usize> = ids
                            .iter()
                            .filter_map(|hid| {
                                hid.split_once(':')
                                    .and_then(|(p, idx)| {
                                        if p == path.as_str() {
                                            idx.parse().ok()
                                        } else {
                                            None
                                        }
                                    })
                            })
                            .collect();
                        if filtered.is_empty() {
                            None
                        } else {
                            Some(filtered)
                        }
                    });

                for hunk in &file_diff.hunks {
                    if let Some(ref ids) = hunk_ids
                        && !ids.contains(&hunk.id)
                    {
                        continue;
                    }

                    for diff_line in &hunk.lines {
                        let (prefix, line_color) = match diff_line
                            .line_type
                        {
                            DiffLineType::Add => ("+", Color::Green),
                            DiffLineType::Delete => ("-", Color::Red),
                            DiffLineType::Header => ("", Color::Cyan),
                            DiffLineType::None => (" ", Color::Reset),
                        };

                        let text = format!(
                            "{}{}",
                            prefix, diff_line.content
                        );

                        if allow_colors && line_color != Color::Reset
                        {
                            lines.push(format!(
                                "{}",
                                text.with(line_color)
                            ));
                        } else {
                            lines.push(text);
                        }
                    }
                }
            }
        }

        lines.push(String::new());
    }

    lines
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
