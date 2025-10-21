use crate::consts::COMMIT_CONVENTION;

pub fn build_prompt(
    use_convention: bool,
    sys_prompt: &str,
    rules: &str,
    stage_hunks: bool,
) -> String {
    let mut prompt = String::new();

    prompt.push_str(sys_prompt);
    prompt.push('\n');

    prompt.push_str(rules);
    prompt.push('\n');

    if use_convention {
        prompt.push_str(COMMIT_CONVENTION);
    }

    prompt.push('\n');

    if stage_hunks {
        prompt.push_str(
        "Fill hunk_ids with the HUNK_ID values shown in the diffs (format: \"filepath:index\").\
        Each hunk can only appear in ONE commit.\
        Ex.: [\"src/main.rs:0\", \"src/git/repo.rs:1\"]",
        );
    } else {
        prompt.push_str(
            "Fill out files with valid paths and leave hunk_headers empty",
        );
    }

    prompt.push('\n');

    prompt
}
