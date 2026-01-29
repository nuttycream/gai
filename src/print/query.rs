use console::style;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

use super::InputHistory;

/// Input prompt
pub fn print_input_prompt(
    prompt: &str,
    history: Option<&mut InputHistory>,
) -> anyhow::Result<Option<String>> {
    let s = match history {
        Some(h) => {
            println!(
                "Type {}/{}/{} to exit. Press {} to show session queries.",
                style("exit")
                    .red()
                    .bold(),
                style("quit")
                    .red()
                    .bold(),
                style("q")
                    .red()
                    .bold(),
                style("Up")
                    .blue()
                    .bold()
            );

            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .history_with(h)
                .interact_text()?
                .to_string()
        }
        None => {
            println!(
                "Type {}/{}/{} to exit.",
                style("exit")
                    .red()
                    .bold(),
                style("quit")
                    .red()
                    .bold(),
                style("q")
                    .red()
                    .bold()
            );
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt)
                .interact_text()?
                .to_string()
        }
    };

    if s == "exit" || s == "q" || s == "quit" {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

// prints multiple choice prompt
// does not require to add Exit to
// option list, if Exit is selected
// returns None
pub fn print_choice_prompt(
    options: &[&str],
    default: Option<usize>,
    prompt: Option<&str>,
) -> anyhow::Result<Option<usize>> {
    let mut options = options.to_vec();

    options.push("Exit");

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.unwrap_or("Select an Option:"))
        .items(&options)
        .default(default.unwrap_or(0))
        .interact()?;

    if selection == options.len() - 1 {
        return Ok(None);
    }

    Ok(Some(selection))
}

pub fn print_retry_prompt(
    prompt: Option<&str>
) -> anyhow::Result<bool> {
    let selection = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.unwrap_or("Retry?"))
        .interact()?;

    Ok(selection)
}
