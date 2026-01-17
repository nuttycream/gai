use console::style;
use dialoguer::{Input, theme::ColorfulTheme};

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
    prompt: &str,
    choices: &[String],
) -> anyhow::Result<Option<usize>> {
    Ok(None)
}
