use std::collections::HashMap;

use git2::{DiffHunk, DiffLine, DiffOptions, Repository};

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
    pub header: String,

    /// use difftype as the key,
    /// with a list of lines
    pub line_diffs: HashMap<DiffType, Vec<String>>,
}

/// taken from diffline::origin
#[derive(Default, Debug, Eq, Hash, PartialEq)]
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
            let path = delta
                .new_file()
                .path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            let diff_hunks =
                self.diffs.entry(path).or_insert_with(Vec::new);

            process_file_diff(diff_hunks, &hunk, &line);

            true
        })?;

        println!("{:#?}", self.diffs);

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

        let find_hunk =
            diff_hunks.iter_mut().find(|hunk| hunk.header == header);

        match find_hunk {
            Some(existing_hunk) => {
                existing_hunk
                    .line_diffs
                    .entry(diff_type)
                    .or_insert_with(Vec::new)
                    .push(content);
            }
            None => {
                let mut line_diffs = HashMap::new();
                line_diffs.insert(diff_type, vec![content]);

                let new_hunk = HunkDiff { header, line_diffs };
                diff_hunks.push(new_hunk);
            }
        }
    }
}
