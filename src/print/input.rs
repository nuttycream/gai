use std::io::{Write, stdin};
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

use super::renderer::Renderer;

pub(crate) fn prompt(
    renderer: &Renderer,
    prompt: &str,
) -> anyhow::Result<String> {
    let mut out = StandardStream::stdout(ColorChoice::Auto);

    if renderer
        .style
        .allow_colors
    {
        out.set_color(
            ColorSpec::new().set_fg(Some(
                renderer
                    .style
                    .highlight,
            )),
        )?;
    }

    write!(out, "{}", &prompt)?;

    out.reset()?;

    out.flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    Ok(input
        .trim_end()
        .to_string())
}
