use std::io::stdout;

use crossterm::{
    execute,
    style::{
        Attribute, Color, ContentStyle, Print, PrintStyledContent,
        Stylize,
    },
};

use crate::{
    git::status::{FileStatus, StatusItemType},
    providers::provider::{ProviderKind, ProviderSettings},
};

use super::{
    renderer::Renderer,
    tree::{Tree, TreeItem},
};

pub fn provider_info(
    renderer: &Renderer,
    provider: &ProviderKind,
    provider_settings: &ProviderSettings,
) -> anyhow::Result<()> {
    let mut out = stdout();
    let model = provider_settings.get_model(provider);
    execute!(
        out,
        Print("Active Provider: "),
        PrintStyledContent(
            provider
                .to_string()
                .with(Color::Green)
        ),
        Print("\r\n"),
        Print("Active Model: "),
        PrintStyledContent(model.with(Color::Yellow)),
        Print("\r\n")
    )?;

    Ok(())
}

pub fn repo_status(
    branch: &str,
    staged_statuses: &[FileStatus],
    working_dir_statuses: &[FileStatus],
    compact: bool,
) -> anyhow::Result<()> {
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
                )
                .style(ContentStyle::new().with(Color::Red));
                new.push(item);
            }
            StatusItemType::Modified => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                )
                .style(ContentStyle::new().with(Color::Yellow));
                modified.push(item);
            }
            StatusItemType::Deleted => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                )
                .style(
                    ContentStyle::new()
                        .with(Color::Red)
                        .attribute(Attribute::Dim),
                );
                deleted.push(item);
            }
            StatusItemType::Renamed => {
                let item = TreeItem::new_leaf(
                    status.path.clone(),
                    &status.path,
                )
                .style(ContentStyle::new().with(Color::Cyan));
                renamed.push(item);
            }
            _ => {}
        }
    }

    let mut unstaged = Vec::new();

    if !modified.is_empty() {
        let count = modified.len();
        unstaged.push(
            TreeItem::new(
                "modified".to_owned(),
                format!("Modified ({})", count),
                modified,
            )?
            .style(
                ContentStyle::new()
                    .with(Color::Yellow)
                    .attribute(Attribute::Bold),
            ),
        );
    }

    if !deleted.is_empty() {
        let count = deleted.len();
        unstaged.push(
            TreeItem::new(
                "deleted".to_owned(),
                format!("Deleted ({})", count),
                deleted,
            )?
            .style(
                ContentStyle::new()
                    .with(Color::Red)
                    .attribute(Attribute::Dim)
                    .attribute(Attribute::Bold),
            ),
        );
    }

    if !renamed.is_empty() {
        let count = renamed.len();
        unstaged.push(
            TreeItem::new(
                "renamed".to_owned(),
                format!("Renamed ({})", count),
                renamed,
            )?
            .style(
                ContentStyle::new()
                    .with(Color::Cyan)
                    .attribute(Attribute::Bold),
            ),
        );
    }

    if !new.is_empty() {
        let count = new.len();
        unstaged.push(
            TreeItem::new(
                "untracked".to_owned(),
                format!("Untracked ({})", count),
                new,
            )?
            .style(
                ContentStyle::new()
                    .with(Color::Red)
                    .attribute(Attribute::Bold),
            ),
        );
    }

    let mut staged_items = Vec::new();

    for status in staged_statuses {
        let item =
            TreeItem::new_leaf(status.path.clone(), &status.path)
                .style(ContentStyle::new().with(Color::Green));
        staged_items.push(item);
    }

    let mut root_items = Vec::new();

    if !staged_items.is_empty() {
        let count = staged_items.len();
        root_items.push(
            TreeItem::new(
                "staged".to_owned(),
                format!("Staged Changes ({})", count),
                staged_items,
            )?
            .style(
                ContentStyle::new()
                    .with(Color::Green)
                    .attribute(Attribute::Bold),
            ),
        );
    }

    if !unstaged.is_empty() {
        let count: usize = unstaged
            .iter()
            .map(|c| c.children().len())
            .sum();
        root_items.push(
            TreeItem::new(
                "unstaged".to_owned(),
                format!("Unstaged Changes ({})", count),
                unstaged,
            )?
            .style(
                ContentStyle::new()
                    .with(Color::Yellow)
                    .attribute(Attribute::Bold),
            ),
        );
    }

    let branch_display = ContentStyle::new()
        .with(Color::Cyan)
        .apply(branch);

    execute!(
        stdout(),
        Print("On Branch: "),
        PrintStyledContent(branch_display),
        Print("\r\n"),
    )?;

    if !root_items.is_empty() {
        Tree::new(&root_items)?
            .collapsed(compact)
            .style(ContentStyle::new().attribute(Attribute::Dim))
            .render();
    }

    Ok(())
}
