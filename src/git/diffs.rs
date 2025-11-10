use git2::{DiffHunk, DiffLine, DiffOptions};
use walkdir::WalkDir;

use crate::git::repo::{
    DiffType, GaiFile, GaiGit, HunkDiff, LineDiff,
};

impl GaiGit {
    pub fn create_diffs(
        &mut self,
        files_to_truncate: &[String],
    ) -> Result<(), git2::Error> {
        // start this puppy up
        let mut opts = DiffOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .enable_fast_untracked_dirs(true);

        let repo = &self.repo;

        let head = repo.head()?.peel_to_tree()?;
        let diff =
            repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;

        let mut gai_files: Vec<GaiFile> = Vec::new();

        diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            let path = delta
                .new_file()
                .path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            let should_truncate =
                files_to_truncate.iter().any(|f| path.ends_with(f));

            let gai_file =
                match gai_files.iter_mut().find(|g| g.path == path) {
                    Some(existing) => existing,
                    None => {
                        gai_files.push(GaiFile {
                            path: path.clone(),
                            should_truncate,
                            hunks: Vec::new(),
                        });
                        gai_files.last_mut().unwrap()
                    }
                };

            process_file_diff(&mut gai_file.hunks, &hunk, &line);

            true
        })?;

        self.files = gai_files;

        // handle untracked files here
        for path in &self.status.u_new {
            let should_truncate =
                files_to_truncate.iter().any(|f| path.ends_with(f));

            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().is_file()
                    && let Ok(content) =
                        std::fs::read_to_string(entry.path())
                {
                    let path = entry.path().to_str().unwrap();
                    let lines: Vec<LineDiff> = content
                        .lines()
                        .map(|line| LineDiff {
                            diff_type: DiffType::Additions,
                            content: format!("{}\n", line),
                        })
                        .collect();

                    self.files.push(GaiFile {
                        path: path.to_owned(),
                        should_truncate,
                        hunks: vec![HunkDiff {
                            header: format!(
                                "New File {}",
                                lines.len()
                            ),
                            line_diffs: lines,
                        }],
                    });
                }
            }
        }

        self.files.sort_by_key(|g| g.should_truncate);

        Ok(())
    }
}

fn process_file_diff(
    diff_hunks: &mut Vec<HunkDiff>,
    hunk: &Option<DiffHunk>,
    line: &DiffLine,
) {
    if let Some(h) = hunk {
        let header = str::from_utf8(h.header())
            .unwrap_or("not a valid utf8 header from hunk")
            .to_owned();
        let content = str::from_utf8(line.content())
            .unwrap_or("not a valid utf8 line from hunk")
            .to_owned();

        let diff_type = match line.origin() {
            '+' => DiffType::Additions,
            '-' => DiffType::Deletions,
            ' ' => DiffType::Unchanged,
            _ => return,
        };

        let diff_line = LineDiff { diff_type, content };

        // instead of storing the different types.
        // we can just push line diffs in a clear order
        // if i want to filter it out, i can do that
        // later, this should just care about the diff itself
        match diff_hunks.iter_mut().find(|h| h.header == header) {
            Some(existing) => existing.line_diffs.push(diff_line),
            None => {
                diff_hunks.push(HunkDiff {
                    header,
                    line_diffs: vec![diff_line],
                });
            }
        }
    }
}
