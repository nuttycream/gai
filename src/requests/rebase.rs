use crate::settings::Settings;

use super::Request;

pub fn create_rebase_request(
    settings: &Settings,
    git_logs: &[String],
) -> Request {
    let prompt = build_prompt(settings);

    Request::new(prompt).insert_contents(git_logs)
}

fn build_prompt(_cfg: &Settings) -> &str {
    "You are a Git expert tasked with rewriting commits. \
        You are supplied a diff and the past commit messages."
}
