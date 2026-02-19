use crate::settings::Settings;

use super::Request;

pub fn create_rebase_plan_request(
    settings: &Settings,
    git_logs: &[String],
    diffs: &str,
) -> Request {
    let prompt = build_prompt(settings);

    Request::new(prompt)
        .insert_contents(git_logs)
        .insert_content(diffs)
}

fn build_prompt(_cfg: &Settings) -> &str {
    "You are a Git master.\
    You are currently tasked with creating a Rebase Plan in the style of an --edit-todo"
}
