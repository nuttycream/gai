use std::{cell::RefCell, fmt, path::Path, rc::Rc};

use git2::{
    Delta, Diff, DiffDelta, DiffFormat, DiffHunk, Oid, Patch,
    Repository,
};

use crate::git::{
    branch::get_head_oid,
    commit::{OldNew, get_compare_commits_diff},
};

use super::{
    errors::GitError,
    status::StatusStrategy,
    status::get_status,
    utils::{get_head_repo, is_newline, new_file_content},
};

// populated after
// loading config
// NOT NEEDED FOR SETTINGs
// but can be modified
// dont think passing around
// config is needed for this case
/// diffing strategy
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DiffStrategy {
    /// send the diffs with the
    /// staged files ONLy
    pub status_strategy: StatusStrategy,

    /// files to truncate
    /// will show as
    /// "TRUNCATED FILE"
    /// ideally this could be set
    /// automatically
    pub truncated_files: Vec<String>,

    /// files to ignore separate
    /// from .gitignore
    pub ignored_files: Vec<String>,
}

/// diff set
#[derive(Clone, Debug, Default)]
pub struct Diffs {
    pub files: Vec<FileDiff>,
}

/// helper struct for ez LLM hunk
/// designation, instead of copying
/// entire hunk headers, hunks are ordered
/// as they are found within a file
/// this converts to src/main.rs:0 for the
/// first hunk in a src/main.rs diff
pub struct HunkId {
    pub path: String,
    pub index: usize,
}

#[derive(Debug, Default, Clone)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<Hunk>,
    pub lines: usize,
    pub untracked: bool,
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub id: usize,
    pub header: HunkHeader,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HunkHeader {
    // copied from DiffHunk
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    // full raw header
    //raw: String,
}

/// type of diff of a single line
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash, Debug)]
pub enum DiffLineType {
    /// just surrounding line, no change
    #[default]
    None,
    /// header of the hunk
    Header,
    /// line added
    Add,
    /// line deleted
    Delete,
}

#[derive(Clone, Copy, Default, Hash, Debug, PartialEq, Eq)]
pub struct DiffLinePosition {
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Default, Clone, Hash, Debug)]
pub struct DiffLine {
    pub content: Box<str>,
    pub line_type: DiffLineType,
    pub position: DiffLinePosition,
}

impl From<git2::DiffLineType> for DiffLineType {
    fn from(line_type: git2::DiffLineType) -> Self {
        match line_type {
            git2::DiffLineType::HunkHeader => Self::Header,
            git2::DiffLineType::DeleteEOFNL
            | git2::DiffLineType::Deletion => Self::Delete,
            git2::DiffLineType::AddEOFNL
            | git2::DiffLineType::Addition => Self::Add,
            _ => Self::None,
        }
    }
}

impl fmt::Display for DiffLineType {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let prefix = match self {
            DiffLineType::None => " ",
            DiffLineType::Header => "",
            DiffLineType::Add => "+",
            DiffLineType::Delete => "-",
        };

        write!(f, "{}", prefix)
    }
}

impl PartialEq<&git2::DiffLine<'_>> for DiffLinePosition {
    fn eq(
        &self,
        other: &&git2::DiffLine,
    ) -> bool {
        other.new_lineno() == self.new_lineno
            && other.old_lineno() == self.old_lineno
    }
}

impl From<&git2::DiffLine<'_>> for DiffLinePosition {
    fn from(line: &git2::DiffLine<'_>) -> Self {
        Self {
            old_lineno: line.old_lineno(),
            new_lineno: line.new_lineno(),
        }
    }
}

impl From<DiffHunk<'_>> for HunkHeader {
    fn from(h: DiffHunk) -> Self {
        Self {
            old_start: h.old_start(),
            old_lines: h.old_lines(),
            new_start: h.new_start(),
            new_lines: h.new_lines(),
            /* raw: String::from_utf8(h.header().to_vec())
            .unwrap_or_default(), */
        }
    }
}

impl TryFrom<&str> for HunkId {
    type Error = GitError;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        let (path, index) = v
            .split_once(':')
            .ok_or_else(|| GitError::InvalidHunk(v.to_owned()))?;

        let path = path.to_owned();
        let index = index
            .parse()
            .map_err(|_| GitError::InvalidHunk(v.to_owned()))?;

        Ok(Self { path, index })
    }
}

impl Diffs {
    /// helper to returns diffs as a list of files
    pub fn as_files(&self) -> Vec<String> {
        let mut vec = Vec::new();

        for diff in &self.files {
            vec.push(diff.path.to_owned());
        }

        vec
    }

    /// helper to return diffs as a list of hunk id strings
    /// format: file:index
    pub fn as_hunks(&self) -> Vec<String> {
        let mut vec = Vec::new();

        for diff in &self.files {
            for hunk in &diff.hunks {
                vec.push(format!("{}:{}", diff.path, hunk.id));
            }
        }

        vec
    }
}

/// helper for converting into a string
/// the LLM request
impl From<Diffs> for String {
    fn from(value: Diffs) -> Self {
        let mut s = String::new();

        for file in &value.files {
            let mut f_str = String::new();
            for hunk in file.hunks.iter() {
                f_str.push_str(&format!(
                    "HunkId[{}:{}]\n",
                    file.path, hunk.id
                ));

                /* f_str.push_str(&hunk.header.raw);
                f_str.push('\n'); */

                for line in &hunk.lines {
                    f_str.push_str(&format!(
                        "{}{}",
                        line.line_type, line.content
                    ));
                    f_str.push('\n');
                }
            }
            s.push_str(&f_str);
            s.push('\n');
        }

        s
    }
}

/// helper for printing for
/// the LLM request
impl fmt::Display for Diffs {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut s = String::new();

        for file in &self.files {
            let mut f_str = String::new();
            for hunk in file.hunks.iter() {
                f_str.push_str(&format!(
                    "HunkId[{}:{}]\n",
                    file.path, hunk.id
                ));

                /* f_str.push_str(&hunk.header.raw);
                f_str.push('\n'); */

                for line in &hunk.lines {
                    f_str.push_str(&format!(
                        "{}{}",
                        line.line_type, line.content
                    ));
                    f_str.push('\n');
                }
            }
            s.push_str(&f_str);
            s.push('\n');
        }

        write!(f, "{}", s)
    }
}

/// helper to remove specified
/// hunks from a specific
/// file diff
pub fn remove_hunks(
    file_diffs: &mut [FileDiff],
    file_path: &str,
    used_ids: &[usize],
) {
    if let Some(file_diff) = file_diffs
        .iter_mut()
        .find(|f| f.path == file_path)
    {
        file_diff
            .hunks
            .retain(|hunk| !used_ids.contains(&hunk.id));
    }
}

/// helper to find a specific file_diff
pub fn find_file_diff<'a>(
    og_file_diffs: &'a [FileDiff],
    file_path: &str,
) -> anyhow::Result<&'a FileDiff> {
    og_file_diffs
        .iter()
        .find(|f| f.path == file_path)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "{} is not in the og_file_diffs",
                file_path
            )
        })
}

/// helper to find file_hunks
/// using a lsit of hunk_ids
/// indices
pub fn find_file_hunks(
    file_diff: &FileDiff,
    ids: Vec<usize>,
) -> anyhow::Result<Vec<Hunk>> {
    let mut hunks = Vec::new();

    for hunk in file_diff
        .hunks
        .clone()
    {
        if ids.contains(&hunk.id) {
            hunks.push(hunk);
        }
    }

    if hunks.is_empty() {
        return Err(GitError::Generic(format!(
            "no matching hunks found in {}",
            file_diff.path
        ))
        .into());
    }

    Ok(hunks)
}

/// build a list of FileDiff's
/// using DiffStrategy
/// calls get_status() first
pub fn get_diffs_from_statuses(
    repo: &Repository,
    work_dir: &Path,
    strategy: &DiffStrategy,
) -> anyhow::Result<Diffs> {
    let mut files = Vec::new();

    let status = get_status(repo, &strategy.status_strategy)?;

    for file in status.statuses {
        let raw_diff =
            get_diff_raw_from_statuses(repo, &file.path, strategy)?;

        let file_diff =
            raw_diff_to_file_diff(&raw_diff, &file.path, work_dir)?;

        files.push(file_diff);
    }

    Ok(Diffs { files })
}

/// builds a list of FileDiffs
/// from specified Oid, can use
/// an optional to Oid, if None
/// is supplied, will use the head
/// of current branch
pub fn get_diffs_from_commits(
    repo: &Repository,
    work_dir: &Path,
    from: Oid,
    to: Option<Oid>,
) -> anyhow::Result<Diffs> {
    let mut files = Vec::new();

    let head = if let Some(to) = to {
        to
    } else {
        get_head_oid(repo, None)?
    };

    let raw_diff = get_compare_commits_diff(
        repo,
        OldNew {
            old: from,
            new: head,
        },
    )?;

    // collect diffs from each file
    for delta in raw_diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| {
                delta
                    .old_file()
                    .path()
            })
            .map(|p| {
                p.to_string_lossy()
                    .to_string()
            })
            .unwrap_or_default();

        let file_diff =
            raw_diff_to_file_diff(&raw_diff, &path, work_dir)?;

        files.push(file_diff);
    }

    Ok(Diffs { files })
}

/// helper to fill out the valid schema options
pub fn get_hunk_ids(file_diffs: &[FileDiff]) -> Vec<HunkId> {
    let mut hunk_ids = Vec::new();

    for file_diff in file_diffs.iter() {
        for hunk in file_diff
            .hunks
            .iter()
        {
            let hunk = HunkId {
                path: file_diff
                    .path
                    .to_owned(),
                index: hunk.id,
            };

            hunk_ids.push(hunk);
        }
    }

    hunk_ids
}

fn get_diff_raw_from_statuses<'a>(
    repo: &'a Repository,
    path: &str,
    strategy: &DiffStrategy,
) -> anyhow::Result<Diff<'a>> {
    let mut opt = git2::DiffOptions::new();

    opt.pathspec(path);

    let diff = match strategy.status_strategy {
        StatusStrategy::Stage => {
            // diff against head
            if let Ok(id) = get_head_repo(repo) {
                let parent = repo.find_commit(id)?;

                let tree = parent.tree()?;
                repo.diff_tree_to_index(
                    Some(&tree),
                    Some(&repo.index()?),
                    Some(&mut opt),
                )?
            } else {
                repo.diff_tree_to_index(
                    None,
                    Some(&repo.index()?),
                    Some(&mut opt),
                )?
            }
        }
        StatusStrategy::WorkingDir => {
            opt.include_untracked(true);
            opt.recurse_untracked_dirs(true);
            repo.diff_index_to_workdir(None, Some(&mut opt))?
        }
        StatusStrategy::Both => {
            if let Ok(id) = get_head_repo(repo) {
                let parent = repo.find_commit(id)?;
                let tree = parent.tree()?;
                opt.include_untracked(true);
                opt.recurse_untracked_dirs(true);
                repo.diff_tree_to_workdir(
                    Some(&tree),
                    Some(&mut opt),
                )?
            } else {
                opt.include_untracked(true);
                opt.recurse_untracked_dirs(true);
                repo.diff_tree_to_workdir(None, Some(&mut opt))?
            }
        }
    };

    Ok(diff)
}

// use original asyncgit to read
// diff per file then filter/process
// todo process all diffs together
// filter as you come acorss
pub fn raw_diff_to_file_diff(
    diff: &Diff,
    path: &str,
    work_dir: &Path,
) -> anyhow::Result<FileDiff> {
    let res = Rc::new(RefCell::new(FileDiff {
        path: path.to_owned(),
        ..Default::default()
    }));
    {
        let mut current_lines = Vec::new();
        let mut current_hunk: Option<HunkHeader> = None;

        let res_cell = Rc::clone(&res);
        let adder = move |header: &HunkHeader,
                          lines: &Vec<DiffLine>| {
            let mut res = res_cell.borrow_mut();
            let id = res.hunks.len();
            res.hunks
                .push(Hunk {
                    id,
                    header: header.to_owned(),
                    lines: lines.to_owned(),
                });
            res.lines += lines.len();
        };

        let mut put = |_: DiffDelta,
                       hunk: Option<DiffHunk>,
                       line: git2::DiffLine| {
            if let Some(hunk) = hunk {
                let hunk_header = HunkHeader::from(hunk);

                match current_hunk {
                    None => current_hunk = Some(hunk_header),
                    Some(h) => {
                        if h != hunk_header {
                            adder(&h, &current_lines);
                            current_lines.clear();
                            current_hunk = Some(hunk_header);
                        }
                    }
                }

                let diff_line = DiffLine {
                    position: DiffLinePosition::from(&line),
                    content: String::from_utf8_lossy(line.content())
                        //Note: trim await trailing newline characters
                        .trim_matches(is_newline)
                        .into(),
                    line_type: line
                        .origin_value()
                        .into(),
                };

                current_lines.push(diff_line);
            }
        };

        let new_file_diff = if diff.deltas().len() == 1 {
            if let Some(delta) = diff.deltas().next() {
                if delta.status() == Delta::Untracked {
                    let relative_path = delta
                        .new_file()
                        .path()
                        .ok_or_else(|| {
                            GitError::Generic(
                                "new file path is unspecified."
                                    .to_string(),
                            )
                        })?;

                    let newfile_path = work_dir.join(relative_path);

                    if let Some(newfile_content) =
                        new_file_content(&newfile_path)
                    {
                        let mut patch = Patch::from_buffers(
                            &[],
                            None,
                            newfile_content.as_slice(),
                            Some(&newfile_path),
                            None,
                        )?;

                        patch.print(
							&mut |delta,
							      hunk: Option<DiffHunk>,
							      line: git2::DiffLine| {
								put(delta, hunk, line);
								true
							},
						)?;

                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !new_file_diff {
            diff.print(
                DiffFormat::Patch,
                move |delta, hunk, line: git2::DiffLine| {
                    put(delta, hunk, line);
                    true
                },
            )?;
        }

        if !current_lines.is_empty() {
            adder(
                &current_hunk.map_or_else(
                    || {
                        Err(GitError::InvalidHunk(
                            "invalid hunk".to_owned(),
                        ))
                    },
                    Ok,
                )?,
                &current_lines,
            );
        }

        if new_file_diff {
            res.borrow_mut()
                .untracked = true;
        }
    }

    let res = Rc::try_unwrap(res).map_err(|_| {
        GitError::Generic("rc unwrap error".to_owned())
    })?;

    Ok(res.into_inner())
}

/* // for tracked files
fn create_file_diff() -> anyhow::Result<FileDiff> {
    //let mut patch = Patch::from_blob()

    todo!()
}

// for untracked files
fn create_new_file_diff() -> anyhow::Result<FileDiff> {
    todo!()
} */
