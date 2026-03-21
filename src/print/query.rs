use super::InputHistory;

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
