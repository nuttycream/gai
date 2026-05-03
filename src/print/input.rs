use std::io::{Write, stdin, stdout};

pub(crate) fn prompt(prompt: &str) -> anyhow::Result<String> {
    let mut out = stdout();

    write!(out, "{}", &prompt)?;

    out.flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    Ok(input
        .trim_end()
        .to_string())
}
