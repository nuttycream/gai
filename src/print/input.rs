use std::io::{Write, stdin, stdout};

use super::renderer::Renderer;

pub(crate) fn prompt(
    _renderer: &Renderer,
    prompt: &str,
) -> anyhow::Result<String> {
    let mut out = stdout();

    write!(out, "{}", &prompt)?;

    out.flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    Ok(input
        .trim_end()
        .to_string())
}
