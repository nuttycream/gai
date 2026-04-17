use std::io::Write;
use termcolor::{ColorChoice, StandardStream};

use crate::{
    git::status::{FileStatus, StatusItemType},
    providers::provider::{ProviderKind, ProviderSettings},
};

use super::tree::{Tree, TreeItem};

pub fn provider_info(
    provider: &ProviderKind,
    provider_settings: &ProviderSettings,
) -> anyhow::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let model = provider_settings.get_model(provider);

    writeln!(stdout, "Provider: {provider}\nModel: {model}",)?;

    Ok(())
}

pub fn repo_status(
    branch: &str,
    staged_statuses: &[FileStatus],
    working_dir_statuses: &[FileStatus],
    compact: bool,
) -> anyhow::Result<()> {
    let mut out = StandardStream::stdout(ColorChoice::Always);

    let mut modified = Vec::new();
    let mut new = Vec::new();
    let mut deleted = Vec::new();
    let mut renamed = Vec::new();

    for status in working_dir_statuses {
        match status.status {
            StatusItemType::New => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                );
                new.push(item);
            }
            StatusItemType::Modified => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                );
                modified.push(item);
            }
            StatusItemType::Deleted => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                );
                deleted.push(item);
            }
            StatusItemType::Renamed => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                );
                renamed.push(item);
            }
            _ => {}
        }
    }

    let mut unstaged = Vec::new();

    if !modified.is_empty() {
        let count = modified.len();
        unstaged.push(TreeItem::new(
            "modified".to_owned(),
            format!("Modified ({})", count),
            modified,
        )?);
    }

    if !deleted.is_empty() {
        let count = deleted.len();
        unstaged.push(TreeItem::new(
            "deleted".to_owned(),
            format!("Deleted ({})", count),
            deleted,
        )?);
    }

    if !renamed.is_empty() {
        let count = renamed.len();
        unstaged.push(TreeItem::new(
            "renamed".to_owned(),
            format!("Renamed ({})", count),
            renamed,
        )?);
    }

    if !new.is_empty() {
        let count = new.len();
        unstaged.push(TreeItem::new(
            "untracked".to_owned(),
            format!("Untracked ({})", count),
            new,
        )?);
    }

    let mut staged_items = Vec::new();

    for status in staged_statuses {
        let item =
            TreeItem::new_leaf(status.path.clone(), &status.path);
        staged_items.push(item);
    }

    let mut root_items = Vec::new();

    if !staged_items.is_empty() {
        let count = staged_items.len();
        root_items.push(TreeItem::new(
            "staged".to_owned(),
            format!("Staged Changes ({})", count),
            staged_items,
        )?);
    }

    if !unstaged.is_empty() {
        let count: usize = unstaged
            .iter()
            .map(|c| c.children().len())
            .sum();
        root_items.push(TreeItem::new(
            "unstaged".to_owned(),
            format!("Unstaged Changes ({})", count),
            unstaged,
        )?);
    }

    writeln!(out, "On branch: {}", branch,)?;

    if !root_items.is_empty() {
        Tree::new(&root_items)?
            .collapsed(compact)
            .render(&mut out);
    }

    Ok(())
}
