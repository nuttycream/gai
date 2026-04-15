use crossterm::{
    queue,
    style::{Print, ResetColor, SetForegroundColor, Stylize},
};
use std::io::{Write, stdin, stdout};

use super::renderer::Renderer;

#[derive(Debug)]
pub(crate) enum InputType {
    Text(String),
    Number(usize),
    None,
}

/// fuzzy finds from your input options and returns
/// the index for the top most matching query
/// shows a preview on the line above.
pub(crate) fn fuzzy_to_idx(
    renderer: &Renderer,
    prompt: &str,
    options: &[String],
) -> anyhow::Result<InputType> {
    Ok(InputType::None)
}

fn draw_input(
    renderer: &Renderer,
    out: &mut impl Write,
    prompt: &str,
    buf: &str,
    options: &[String],
) -> std::io::Result<()> {
    Ok(())
}

pub(crate) fn prompt(
    renderer: &Renderer,
    prompt: &str,
) -> anyhow::Result<String> {
    let mut out = stdout();

    if renderer
        .style
        .allow_colors
    {
        queue!(
            out,
            SetForegroundColor(
                renderer
                    .style
                    .highlight
            )
        )?;
    }

    queue!(out, Print(&prompt), ResetColor)?;

    out.flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    Ok(input
        .trim_end()
        .to_string())
}

// prints multiple choice prompt
// does not require to add Exit to
// option list, if Exit is selected
// returns None
pub fn option_prompt(
    options: &[&str],
    default: Option<usize>,
    prompt: Option<&str>,
) -> anyhow::Result<Option<usize>> {
    Ok(None)
}

pub fn retry_prompt(prompt: Option<&str>) -> anyhow::Result<bool> {
    Ok(false)
}
