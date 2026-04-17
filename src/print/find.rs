use termcolor::{Color, ColorChoice, ColorSpec, StandardStream};

use crate::{git::log::GitLog, schema::find::Confidence};

use super::{renderer::Renderer, tree::Tree, tree::TreeItem};

pub fn found_commit(
    renderer: &Renderer,
    commit: &GitLog,
    _reasoning: &str,
    confidence: Confidence,
) -> anyhow::Result<()> {
    let mut out = StandardStream::stdout(ColorChoice::Auto);

    let allow_colors = renderer
        .style
        .allow_colors;

    let _conf_color = if allow_colors {
        match confidence {
            Confidence::Exact => Color::Green,
            Confidence::Likely => Color::Yellow,
            Confidence::Ambiguous => Color::Magenta,
        }
    } else {
        Color::White
    };

    //let commit_color = prefix_color(commit.prefix);
    let mut children = Vec::new();
    let date_item = TreeItem::new_leaf(
        commit
            .date
            .to_owned(),
        format!("Date: {}", commit.date),
    );

    children.push(date_item);

    let author_item = TreeItem::new_leaf(
        commit
            .author
            .to_owned(),
        format!("Author: {}", commit.author),
    );

    children.push(author_item);

    // TODO: show commit stats instead,
    // 1 files changed, 2 insertions, 3 deletions, etc.
    let logs = commit
        .files
        .join(", ");

    let files_item =
        TreeItem::new_leaf("raw_files".to_string(), logs.to_string());

    children.push(files_item);

    let message: String = commit
        .to_owned()
        .into();

    let short_hash = &commit.commit_hash[..7];

    let hash_display = format!("[{}]", short_hash);

    let avail = 70;

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
        .style(ColorSpec::new()),
    ];

    Tree::new(&tree)?.render(&mut out);

    Ok(())
}
