use crate::consts::COMMIT_CONVENTION;

pub fn build_prompt(
    use_convention: bool,
    sys_prompt: &str,
    rules: &str,
) -> String {
    let convention = if use_convention {
        format!("Convention:\n{}", COMMIT_CONVENTION)
    } else {
        "".to_owned()
    };

    format!("{}\nRules:\n{}\n{}", sys_prompt, rules, convention)
}
