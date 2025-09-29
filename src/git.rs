use std::{collections::HashMap, error::Error, path::Path};

use git2::{
    DiffHunk, DiffLine, DiffOptions, Repository, StatusOptions,
};

use crate::response::Commit;

pub struct GaiGit {
    pub repo: Repository,
    /// We can track changes using this
    /// where the key is the filename
    /// (maybe even path?) - for fs::read_to_str
    /// and status if it was changed or not
    /// tracked
    ///
    /// everything else such as unmodified, ignored
    /// doesnt get saved here
    pub file: Vec<String>,

    pub diffs: HashMap<String, Vec<HunkDiff>>,

    options: StatusOptions,
}

#[derive(Debug)]
pub struct HunkDiff {
    /// example key (header)
    /// @@ -12,8 +12,9 @@
    /// since raw line numbers
    /// may be inconsistent
    pub header: String,

    pub line_diffs: Vec<LineDiff>,
}

#[derive(Debug)]
pub struct LineDiff {
    pub diff_type: DiffType,
    pub content: String,
}

/// taken from diffline::origin
#[derive(Default, Debug, Eq, Hash, PartialEq)]
pub enum DiffType {
    #[default]
    Unchanged,
    Additions,
    Deletions,
}

impl GaiGit {
    /// this could fail on an unitialized directory
    /// for now, im not gonna handle those and we
    /// just straight up panic if we failed to open
    pub fn new(repo_path: &Path) -> Result<Self, Box<dyn Error>> {
        let repo = Repository::open(repo_path)?;
        let mut options = StatusOptions::new();

        options.include_untracked(true);

        if repo.statuses(Some(&mut options))?.is_empty() {
            return Err("no diffs".into());
        }

        Ok(GaiGit {
            repo,
            options,
            file: Vec::new(),
            diffs: HashMap::new(),
        })
    }

    pub fn status(
        &mut self,
        to_ignore: &[String],
    ) -> Result<(), Box<dyn Error>> {
        let statuses = self.repo.statuses(Some(&mut self.options))?;

        for entry in statuses.iter() {
            if entry.status() == git2::Status::CURRENT
                || entry.index_to_workdir().is_none()
            {
                continue;
            }

            if to_ignore
                .iter()
                .any(|f| entry.path().unwrap().ends_with(f))
            {
                continue;
            }

            let path = entry.path().unwrap().to_owned();

            self.file.push(path);
        }

        Ok(())
    }

    pub fn create_diffs(
        &mut self,
        to_ignore: &[String],
    ) -> Result<Vec<String>, git2::Error> {
        // start this puppy up
        let mut opts = DiffOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .enable_fast_untracked_dirs(true);

        let repo = &self.repo;

        let head = repo.head()?.peel_to_tree()?;
        let diff =
            repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;

        let mut files = Vec::new();

        diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            match delta.status() {
                // Delta::Added => todo!(),
                // Delta::Deleted => todo!(),
                // Delta::Modified => todo!(),
                // Delta::Renamed => todo!(),
                // Delta::Copied => todo!(),
                // Delta::Ignored => todo!(),
                // Delta::Untracked => todo!(),
                // Delta::Typechange => todo!(),
                // Delta::Unreadable => todo!(),
                // Delta::Conflicted => todo!(),
                // ignore unmodified
                _ => {}
            }

            let path = delta
                .new_file()
                .path()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            // skip I think
            if to_ignore.iter().any(|f| path.ends_with(f)) {
                return true;
            }

            let diff_hunks = self
                .diffs
                .entry(path.clone())
                .or_insert_with(Vec::new);

            process_file_diff(diff_hunks, &hunk, &line);

            files.push(path);

            true
        })?;

        Ok(files)
    }

    pub fn apply_commits(&self, commits: &[Commit]) {
        //println!("{:#?}", self.commits);
        for commit in commits {
            self.commit(commit);
        }
    }

    fn commit(&self, commit: &Commit) {
        let mut index = self.repo.index().unwrap();

        index.clear().unwrap();

        if let Ok(head) = self.repo.head() {
            if let Ok(tree) = head.peel_to_tree() {
                index.read_tree(&tree).unwrap();
            }
        }

        // staging
        for path in &commit.files {
            let path = Path::new(&path);
            let status = self.repo.status_file(path).unwrap();

            if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::WT_NEW)
            {
                index.add_path(path).unwrap();
            }
        }

        index.write().unwrap();

        let tree_oid = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_oid).unwrap();

        let parent_commit = match self.repo.revparse_single("HEAD") {
            Ok(obj) => Some(obj.into_commit().unwrap()),
            // ignore first commit
            Err(_) => None,
        };

        let mut parents = Vec::new();
        if let Some(parent) = parent_commit.as_ref() {
            parents.push(parent);
        }

        let sig = self.repo.signature().unwrap();

        self.repo
            .commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!(
                    "{}: {}",
                    match commit.message.prefix {
                        _ => {
                            let prefix = format!(
                                "{:?}",
                                commit.message.prefix
                            );
                            // todo use cfg setting
                            prefix.to_lowercase()
                        }
                    },
                    commit.message.message
                ),
                &tree,
                &parents[..],
            )
            .unwrap();
    }

    // won't bother with this for now
    //fn new_branch(&self, commit: &Commit) {}
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
