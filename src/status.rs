use crate::{
    git::{
        DiffStrategy, GitRepo, StatusStrategy,
        diffs::get_diffs_from_statuses, status::get_status,
    },
    print::status,
    requests::tokens::estimate_token_count,
    settings::Settings,
};

pub fn run() -> anyhow::Result<()> {
    // let git = GitRepo::open(None)?;
    // let settings = Settings::default();
    //
    // // todo impl something for this
    // // so we dont have to pass in two vectors
    // // into print
    // // likely gonna be handled within git::GitStatus
    // let staged = get_status(&git.repo, &StatusStrategy::Stage)?;
    // let working_dir =
    //     get_status(&git.repo, &StatusStrategy::WorkingDir)?;
    //
    // let provider = settings.provider;
    //
    // status::provider_info(&provider, &settings.providers)?;
    //
    // status::repo_status(
    //     &staged.branch_name,
    //     &staged.statuses,
    //     &working_dir.statuses,
    //     global.compact,
    // )?;
    //
    // if args.verbose {
    //     let mut diff_strategy = DiffStrategy {
    //         ..Default::default()
    //     };
    //
    //     if let Some(ref files_to_truncate) = settings
    //         .context
    //         .truncate_files
    //     {
    //         diff_strategy.truncated_files =
    //             files_to_truncate.to_owned();
    //     }
    //
    //     if let Some(ref files_to_ignore) = settings
    //         .context
    //         .ignore_files
    //     {
    //         diff_strategy.ignored_files = files_to_ignore.to_owned();
    //     }
    //
    //     let diffs = get_diffs_from_statuses(
    //         &git.repo,
    //         &git.workdir,
    //         &diff_strategy,
    //     )?;
    //
    //     for file in &diffs.files {
    //         let mut txt = String::new();
    //
    //         // mimicing what gets sent to the prompt
    //         // ideally, this is done near the request
    //         for hunk in &file.hunks {
    //             txt.push_str(&format!(
    //                 "HunkId[{}:{}]\n",
    //                 file.path, hunk.id
    //             ));
    //
    //             for line in &hunk.lines {
    //                 txt.push_str(&format!(
    //                     "{}{}\n",
    //                     line.line_type, line.content
    //                 ));
    //             }
    //         }
    //
    //         let token_estimate = estimate_token_count(&txt);
    //         // temp println
    //         // TODO: remove, use status::repo_status
    //         println!("file:{} tokens:{}", file.path, token_estimate);
    //     }
    // }

    Ok(())
}
