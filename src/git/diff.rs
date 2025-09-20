use std::collections::HashMap;

use git2::{DiffOptions, Repository};

/// an abstracted ver
/// of the lowlevel impl
/// of diffdelta and diffline
pub struct GitDiff {
    /// storing individual file diffs
    /// with their path as the key
    pub diffs: HashMap<String, Vec<DiffHunk>>,
}

pub struct DiffHunk {
    pub line: u32,
    pub diff_type: DiffType,
}

/// taken from diffline::origin
#[derive(Default)]
pub enum DiffType {
    #[default]
    Unchanged,
    Addition,
    Deletion,
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
            true
        })?;

        Ok(())
    }
}
