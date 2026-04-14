use std::io::stdout;

use crossterm::{
    queue,
    style::{
        Attribute, Color, ContentStyle, Print, PrintStyledContent,
        Stylize,
    },
};

use crate::{git::log::GitLog, schema::find::Confidence};

use super::{
    commits::prefix_color, renderer::Renderer, tree::Tree,
    tree::TreeItem,
};

pub fn found_commit(
    renderer: &Renderer,
    commit: &GitLog,
    reasoning: &str,
    confidence: Confidence,
) -> anyhow::Result<()> {
    let mut out = stdout();

    let allow_colors = renderer
        .style
        .allow_colors;

    let secondary = renderer
        .style
        .secondary;

    let highlight = renderer
        .style
        .highlight;

    let conf_color = if allow_colors {
        match confidence {
            Confidence::Exact => Color::Green,
            Confidence::Likely => Color::Yellow,
            Confidence::Ambiguous => Color::Magenta,
        }
    } else {
        Color::White
    };
    //let commit_color = prefix_color(commit.prefix);

    queue!(
        out,
        Print("\r\n"),
        Print("Found a \""),
        PrintStyledContent(
            confidence
                .to_string()
                .attribute(Attribute::Bold)
                .with(conf_color)
        ),
        Print("\" commit"),
        Print("\r\n"),
        PrintStyledContent("Why?\r\n".with(highlight)),
        PrintStyledContent(reasoning.with(highlight)),
        Print("\r\n"),
    )?;

    let mut children = Vec::new();
    let date_item = TreeItem::new_leaf(
        commit
            .date
            .to_owned(),
        format!("Date: {}", commit.date),
    )
    .style(ContentStyle::new().with(secondary));

    children.push(date_item);

    let author_item = TreeItem::new_leaf(
        commit
            .author
            .to_owned(),
        format!("Author: {}", commit.author),
    )
    .style(ContentStyle::new().with(secondary));

    children.push(author_item);

    // TODO: show commit stats instead,
    // 1 files changed, 2 insertions, 3 deletions, etc.
    let logs = commit
        .files
        .join(", ");

    let files_item =
        TreeItem::new_leaf("raw_files".to_string(), logs.to_string())
            .style(ContentStyle::new().with(secondary));

    children.push(files_item);

    let message: String = commit
        .to_owned()
        .into();

    let short_hash = &commit.commit_hash[..7];

    let hash_display = format!("[{}]", short_hash).with(secondary);

    let avail = crossterm::terminal::size()?.0 as usize;

    let truncated = if message.len() > avail {
        format!("{}...", &message[..avail])
    } else {
        message
    };

    let display = format!("{} {}", hash_display, truncated);

    let tree = vec![
        TreeItem::new(
            commit
                .raw
                .to_string(),
            display.to_string(),
            children,
        )?
        .style(ContentStyle::new()),
    ];

    Tree::new(&tree)?.render();

    Ok(())
}
