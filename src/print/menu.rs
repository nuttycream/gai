use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{
        Attribute, Print, ResetColor, SetBackgroundColor,
        SetForegroundColor, Stylize,
    },
    terminal,
};
use std::io::{Write, stdout};

use super::renderer::Renderer;

#[derive(Debug)]
pub enum MenuChosenOption {
    Selected(usize),
    Cancelled,
}

#[derive(Debug, Clone)]
struct MenuItem {
    label: String,
    keybind: u8,
}

/// draws an inline menu, with crossterm
/// event handling, this is a generic
/// function that should and would be handled
/// by higher level functions
/// can take in a max of 9 options
/// prompt/label is rendered inline if compact
pub(crate) fn inline_menu(
    renderer: &Renderer,
    prompt: &str,
    items: &[&str],
) -> anyhow::Result<MenuChosenOption> {
    // lets just not handle more than 9 options
    // lol, just use an input prompt instead
    if items.len() > 9 {
        anyhow::bail!(
            "inline menus should not be able to handle more than 9 options"
        );
    }

    let parsed = build_items(items);

    let mut selected = 0;

    let mut out = stdout();

    terminal::enable_raw_mode()?;

    // dont show cursor for menus
    execute!(out, cursor::Hide)?;

    draw_inline(renderer, &mut out, prompt, &parsed, selected)?;

    let outcome = loop {
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                // handle cancel
                KeyCode::Esc | KeyCode::Char('q') => {
                    break MenuChosenOption::Cancelled;
                }

                KeyCode::Char('c')
                    if modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    break MenuChosenOption::Cancelled;
                }

                // enter or space key
                // though any input from allowed keys
                // will trigger it to continue
                KeyCode::Enter | KeyCode::Char(' ') => {
                    break MenuChosenOption::Selected(selected);
                }

                // horizon movement
                KeyCode::Left
                | KeyCode::Char('h')
                | KeyCode::BackTab => {
                    selected = selected
                        .checked_sub(1)
                        .unwrap_or(parsed.len() - 1);
                }

                KeyCode::Right
                | KeyCode::Char('l')
                | KeyCode::Tab => {
                    selected = (selected + 1) % parsed.len();
                }

                // allowing vertical vim keys as well
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = selected
                        .checked_sub(1)
                        .unwrap_or(parsed.len() - 1);
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    selected = (selected + 1) % parsed.len();
                }

                // keybind handle
                KeyCode::Char(c) => {
                    if c >= '1' && c <= '9' {
                        let idx = (c as u8 - b'1') as usize;
                        if idx < parsed.len() {
                            break MenuChosenOption::Selected(idx);
                        }
                    }
                }

                _ => {}
            }

            draw_inline(
                renderer, &mut out, prompt, &parsed, selected,
            )?;
        }
    };

    execute!(out, cursor::Show)?;
    terminal::disable_raw_mode()?;

    execute!(out, Print("\r\n"))?;

    Ok(outcome)
}

fn build_items(raw: &[&str]) -> Vec<MenuItem> {
    raw.iter()
        .enumerate()
        .map(|(i, s)| MenuItem {
            label: s.to_string(),
            keybind: b'1' + i as u8,
        })
        .collect()
}

fn draw_inline(
    renderer: &Renderer,
    out: &mut impl Write,
    prompt: &str,
    items: &[MenuItem],
    selected: usize,
) -> std::io::Result<()> {
    let primary = renderer
        .style
        .primary;

    let highlight = renderer
        .style
        .highlight;

    let secondary = renderer
        .style
        .secondary;

    // i believe we need to use queue
    // here since we're writing on the same line
    // and thus avoiding continuously
    // rewriting the same line in an execute call?
    queue!(
        out,
        Print("\r"),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;

    if !renderer.compact {
        queue!(
            out,
            cursor::MoveUp(1),
            terminal::Clear(terminal::ClearType::CurrentLine),
        )?;
    }

    let newl = if renderer.compact { "  " } else { "\r\n" };

    queue!(out, Print(prompt.with(primary)), Print(newl))?;

    for (i, item) in items
        .iter()
        .enumerate()
    {
        let is_active = i == selected;
        let bind = format!("{}", item.keybind as char);

        if is_active {
            queue!(
                out,
                SetBackgroundColor(primary),
                SetForegroundColor(secondary),
                Print("["),
                Print(bind),
                Print("] "),
                Print(
                    item.label
                        .to_owned()
                ),
                ResetColor,
            )?;
        } else {
            queue!(
                out,
                Print("[".with(primary)),
                Print(bind.with(highlight)),
                Print("] ".with(primary)),
                Print(
                    item.label
                        .to_owned()
                        .with(secondary)
                ),
            )?;
        }

        if i < items.len() - 1 {
            queue!(out, Print("  "))?;
        }
    }

    out.flush()
}
