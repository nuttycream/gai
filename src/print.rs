use anyhow::Result;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
};
use std::io::Stdout;

use crate::{
    ai::response::ResponseCommit, config::Config, git::repo::GaiGit,
    graph::Arena,
};

pub fn pretty_print_status(
    stdout: &mut Stdout,
    gai: &GaiGit,
) -> Result<()> {
    let mut arena = Arena::new();

    let branch = &gai.get_branch();
    let status = &gai.status;

    let staged_count = gai.staged_len();
    let unstaged_count = gai.unstaged_len();

    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("On Branch: {}\n", branch).bold()),
        ResetColor
    )?;

    if unstaged_count == 0 && staged_count == 0 {
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("No Diffs".bold()),
            ResetColor
        )?;

        return Ok(());
    }

    if staged_count > 0 {
        let staged_root = arena.new_node("✓ Staged", Color::Green);
        arena.set_count(staged_root, staged_count);

        if !status.s_new.is_empty() {
            let new_node = arena.new_node("New", Color::Green);
            arena.set_count(new_node, status.s_new.len());
            arena.add_child(staged_root, new_node);

            for file in &status.s_new {
                let file_node = arena.new_node(file, Color::Green);
                arena.set_prefix(file_node, "A");
                arena.add_child(new_node, file_node);
            }
        }

        // mod
        if !status.s_modified.is_empty() {
            let modified_node =
                arena.new_node("Modified", Color::Blue);
            arena.set_count(modified_node, status.s_modified.len());
            arena.add_child(staged_root, modified_node);

            for file in &status.s_modified {
                let file_node = arena.new_node(file, Color::Blue);
                arena.set_prefix(file_node, "M");
                arena.add_child(modified_node, file_node);
            }
        }

        // del
        if !status.s_deleted.is_empty() {
            let deleted_node = arena.new_node("Deleted", Color::Red);
            arena.set_count(deleted_node, status.s_deleted.len());
            arena.add_child(staged_root, deleted_node);

            for file in &status.s_deleted {
                let file_node = arena.new_node(file, Color::Red);
                arena.set_prefix(file_node, "D");
                arena.add_child(deleted_node, file_node);
            }
        }

        // ren
        if !status.s_renamed.is_empty() {
            let renamed_node =
                arena.new_node("Renamed", Color::Magenta);
            arena.set_count(renamed_node, status.s_renamed.len());
            arena.add_child(staged_root, renamed_node);

            for (old, new) in &status.s_renamed {
                let label = format!("{} → {}", old, new);
                let file_node = arena.new_node(label, Color::White);
                arena.set_prefix(file_node, "R");
                arena.add_child(renamed_node, file_node);
            }
        }
    }

    if unstaged_count > 0 {
        let unstaged_root =
            arena.new_node("⚠ Unstaged", Color::Yellow);
        arena.set_count(unstaged_root, unstaged_count);

        if !status.u_new.is_empty() {
            let new_node = arena.new_node("New", Color::Green);
            arena.set_count(new_node, status.u_new.len());
            arena.add_child(unstaged_root, new_node);

            for file in &status.u_new {
                let file_node = arena.new_node(file, Color::Green);
                arena.set_prefix(file_node, "?");
                arena.add_child(new_node, file_node);
            }
        }

        if !status.u_modified.is_empty() {
            let modified_node =
                arena.new_node("Modified", Color::Blue);
            arena.set_count(modified_node, status.u_modified.len());
            arena.add_child(unstaged_root, modified_node);

            for file in &status.u_modified {
                let file_node = arena.new_node(file, Color::Blue);
                arena.set_prefix(file_node, "M");
                arena.add_child(modified_node, file_node);
            }
        }

        if !status.u_deleted.is_empty() {
            let deleted_node = arena.new_node("Deleted", Color::Red);
            arena.set_count(deleted_node, status.u_deleted.len());
            arena.add_child(unstaged_root, deleted_node);

            for file in &status.u_deleted {
                let file_node = arena.new_node(file, Color::Red);
                arena.set_prefix(file_node, "D");
                arena.add_child(deleted_node, file_node);
            }
        }

        if !status.u_renamed.is_empty() {
            let renamed_node =
                arena.new_node("Renamed", Color::Magenta);
            arena.set_count(renamed_node, status.u_renamed.len());
            arena.add_child(unstaged_root, renamed_node);

            for (old, new) in &status.u_renamed {
                let label = format!("{} → {}", old, new);
                let file_node = arena.new_node(label, Color::White);
                arena.set_prefix(file_node, "R");
                arena.add_child(renamed_node, file_node);
            }
        }
    }

    arena.print_tree(stdout)?;

    Ok(())
}

pub fn pretty_print_commits(
    stdout: &mut Stdout,
    commits: &[ResponseCommit],
    cfg: &Config,
    gai: &GaiGit,
) -> Result<()> {
    let mut arena = Arena::new();

    for (i, commit) in commits.iter().enumerate() {
        let prefix = commit.get_commit_prefix(
            cfg.gai.commit_config.capitalize_prefix,
            cfg.gai.commit_config.include_scope,
        );

        let commit_root = arena
            .new_node(format!("Commit {}", i + 1), Color::DarkGrey);

        let prefix_node = arena.new_node(prefix, Color::Green);
        arena.add_child(commit_root, prefix_node);

        let header_node = arena.new_node(
            format!("Header: {}", commit.message.header),
            Color::White,
        );
        arena.add_child(commit_root, header_node);

        if !commit.message.body.is_empty() {
            let body_text = arena.truncate(&commit.message.body, 45);
            let body_node = arena.new_node(
                format!("Body: {}", body_text),
                Color::Blue,
            );
            arena.add_child(commit_root, body_node);
        }

        if gai.stage_hunks {
            let hunks_node = arena.new_node(
                format!("Hunks: {:?}", commit.hunk_ids),
                Color::Magenta,
            );
            arena.add_child(commit_root, hunks_node);
        } else {
            let files_parent =
                arena.new_node("Files", Color::Magenta);
            arena.set_count(files_parent, commit.files.len());
            arena.add_child(commit_root, files_parent);

            for file in &commit.files {
                let file_node = arena.new_node(file, Color::White);
                arena.add_child(files_parent, file_node);
            }
        }
    }

    arena.print_tree(stdout)?;

    Ok(())
}
