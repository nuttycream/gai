use std::collections::HashMap;

use git2::{DiffLine, DiffOptions, Repository};

/// an abstracted ver
/// of the lowlevel impl
/// of diffdelta and diffline
pub struct GitDiff {
    /// storing individual file diffs
    /// with their path as the key
    pub diffs: HashMap<String, Vec<HunkDiff>>,
}

#[derive(Debug)]
pub struct HunkDiff {
    /// example key (header)
    /// @@ -12,8 +12,9 @@
    /// since raw line numbers
    /// may be inconsistent
    pub diffy: HashMap<String, Vec<LineDiff>>,
}

#[derive(Debug)]
pub struct LineDiff {
    //pub diffy_v2: HashMap<DiffType, String>,
    pub diff_type: DiffType,
    pub content: String,
}

/// taken from diffline::origin
#[derive(Default, Debug)]
pub enum DiffType {
    #[default]
    Unchanged,
    Additions,
    Deletions,
}

impl GitDiff {
    pub fn new() -> Self {
        Self {
            diffs: HashMap::new(),
        }
    }

    /// only call this on State::Status::Changed;
    pub fn create_diffs(
        &mut self,
        repo: &Repository,
    ) -> Result<(), git2::Error> {
        // start this puppy up
        let mut opts = DiffOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .enable_fast_untracked_dirs(true);

        let head = repo.head()?.peel_to_tree()?;
        let diff =
            repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;

        diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            let path =
                delta.new_file().path().unwrap().to_str().unwrap();

            if self.diffs.contains_key(path) {
                process_file_diff(
                    self.diffs.get_mut(path).unwrap(),
                    &hunk,
                    &line,
                );
            } else {
                self.diffs.insert(path.to_owned(), Vec::new());
            }

            true
        })?;

        println!("{:#?}", self.diffs);

        Ok(())
    }
}
fn process_file_diff(
    diff_hunks: &mut Vec<HunkDiff>,
    hunk: &Option<git2::DiffHunk>,
    line: &DiffLine,
) {
    let mut hunks = Vec::new();
    let mut curr_hunk: Option<HunkDiff> = None;

    if let Some(h) = hunk {
        if let Some(taken) = curr_hunk.take() {
            hunks.push(taken);
        }

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
            _ => DiffType::Unchanged,
        };

        let line_diff = LineDiff { diff_type, content };

        let find_hunk = diff_hunks
            .iter_mut()
            .find(|hunk| hunk.diffy.contains_key(&header));

        match find_hunk {
            Some(hunk) => {
                if let Some(lines) = hunk.diffy.get_mut(&header) {
                    lines.push(line_diff);
                }
            }
            None => {
                let mut new_hunk = HunkDiff {
                    diffy: HashMap::new(),
                };

                new_hunk.diffy.insert(header, vec![line_diff]);

                diff_hunks.push(new_hunk);
            }
        }
    }
}
