// small util to estimate tokens
// naive way, i dont want to pull in
// a heavy crate like tiktoken or
// tokenizer and uses models that may
// not be used in all llm providers

/// estimate token counts
/// using length of text + 3 all over 4
pub fn estimate_token_count(text: &str) -> u32 {
    (text.len() as u32 + 3) / 4
}
