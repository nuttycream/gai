use console::style;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

use super::InputHistory;

/// Input prompt
pub fn print_input_prompt(
    prompt: &str,
    history: &mut InputHistory,
) -> anyhow::Result<Option<String>> {
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

    let s = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .history_with(history)
        .interact_text()?
        .to_string();

    if s == "exit" || s == "q" || s == "quit" {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

pub fn print_choice_prompt(
    options: &[String],
    default: Option<usize>,
    prompt: Option<&str>,
) -> anyhow::Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.unwrap_or("Select an Option:"))
        .items(options)
        .default(default.unwrap_or(0))
        .interact()?;

    Ok(selection)
}

pub fn print_retry_prompt(
    prompt: Option<&str>
) -> anyhow::Result<bool> {
    let selection = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt.unwrap_or("Retry?"))
        .interact()?;

    Ok(selection)
}
