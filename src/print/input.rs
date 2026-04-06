use crossterm::{
    cursor::{self},
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::{Print, Stylize},
    terminal,
};
use std::io::{Write, stdout};

use super::{InputHistory, renderer::Renderer};

#[derive(Debug)]
pub(crate) enum InputType {
    Text(String),
    Number(usize),
    None,
}

pub(crate) fn fuzzy_to_num(
    renderer: &Renderer,
    prompt: &str,
    options: &[String],
) -> anyhow::Result<InputType> {
    let mut out = stdout();
    let mut buf = String::new();

    terminal::enable_raw_mode()?;

    crossterm::execute!(out, Print("\r\n"))?;

    draw_input(renderer, &mut out, prompt, &buf, options)?;

    let outcome = loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = crossterm::event::read()?
        {
            match code {
                KeyCode::Esc => {
                    break InputType::None;
                }
                KeyCode::Char('c')
                    if modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    break InputType::None;
                }
                KeyCode::Enter => {
                    if !buf.is_empty() {
                        if let Some(i) = best_match(&buf, options) {
                            break InputType::Number(i);
                        }
                    }
                }
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                }
                _ => {}
            }

            draw_input(&renderer, &mut out, prompt, &buf, options)?;
        }
    };

    // i feel like there should be an smoother way to clean
    // these us up
    // this one only cleans the input line, it should leave
    // the hint area untouched
    crossterm::execute!(
        out,
        Print("\r"),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;

    terminal::disable_raw_mode()?;

    Ok(outcome)
}

fn draw_input(
    renderer: &Renderer,
    out: &mut impl Write,
    prompt: &str,
    buf: &str,
    options: &[String],
) -> std::io::Result<()> {
    let primary = renderer
        .style
        .primary;

    let highlight = renderer
        .style
        .highlight;

    let allow_color = renderer
        .style
        .allow_colors;

    crossterm::queue!(
        out,
        cursor::MoveUp(1),
        Print("\r"),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;

    let hint = match best_match(buf, options) {
        Some(i) => format!("[{}] {}", i + 1, options[i]),
        None => {
            "type to search... | <ESC> <C-c> to exit | Enter to confirm"
                .to_string()
        }
    };

    if allow_color {
        crossterm::queue!(out, Print(hint.with(highlight)))?;
    } else {
        crossterm::queue!(out, Print(&hint))?;
    }

    crossterm::queue!(
        out,
        Print("\r\n"),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;

    if allow_color {
        crossterm::queue!(
            out,
            Print(prompt.with(primary)),
            Print(buf)
        )?;
    } else {
        crossterm::queue!(out, Print(prompt), Print(buf))?;
    }

    out.flush()
}

/// Input prompt
pub fn input_prompt(
    prompt: &str,
    history: Option<&mut InputHistory>,
) -> anyhow::Result<Option<String>> {
    Ok(None)
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

/// wrapper for fuzzy search
/// this returns the index of the matching
/// query
fn best_match(
    input: &str,
    options: &[String],
) -> Option<usize> {
    if input.is_empty() {
        return None;
    }

    let strs = options
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>();

    let results =
        rust_fuzzy_search::fuzzy_search_best_n(input, &strs, 1);

    results
        .first()
        .map(|(matched, _)| {
            options
                .iter()
                .position(|o| o.as_str() == *matched)
        })?
}
