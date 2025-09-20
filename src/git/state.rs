use std::{collections::HashMap, error::Error, path::Path};

use git2::{Repository, StatusOptions};

pub struct GitState {
    pub repo: Repository,
    options: StatusOptions,

    /// We can track changes using this
    /// where the key is the filename
    /// (maybe even path?) - for fs::read_to_str
    /// and status if it was changed or not
    /// tracked
    ///
    /// everything else such as unmodified, ignored
    /// doesnt get saved here
    pub file: HashMap<String, Status>,
}

#[derive(Debug)]
pub enum Status {
    /// change includes
    /// modified, deleted,
    /// typechanged, renamed
    Changed(String),

    /// files that haven't been
    /// added Status::WT_NEW
    Untracked,
}

impl GitState {
    /// this could fail on an unitialized directory
    /// for now, im not gonna handle those and we
    /// just straight up panic if we failed to open
    pub fn new(repo_path: &Path) -> Result<Self, Box<dyn Error>> {
        let repo = Repository::open(repo_path)?;
        let mut options = StatusOptions::new();

        options.include_untracked(true);

        Ok(GitState {
            repo,
            options,
            file: HashMap::new(),
        })
    }

    pub fn status(
        &mut self,
        to_ignore: &[String],
    ) -> Result<(), Box<dyn Error>> {
        let statuses = self.repo.statuses(Some(&mut self.options))?;

        for entry in statuses.iter() {
            // With `Status::OPT_INCLUDE_UNMODIFIED` (not used in this example)
            // `index_to_workdir` may not be `None` even if there are no differences,
            // in which case it will be a `Delta::Unmodified`.
            if entry.status() == git2::Status::CURRENT
                || entry.index_to_workdir().is_none()
            {
                continue;
            }

            let status = match entry.status() {
                s if s.contains(git2::Status::WT_MODIFIED) => {
                    Status::Changed("modified".to_owned())
                }
                s if s.contains(git2::Status::WT_DELETED) => {
                    Status::Changed("deleted".to_owned())
                }
                s if s.contains(git2::Status::WT_RENAMED) => {
                    Status::Changed("renamed".to_owned())
                }
                s if s.contains(git2::Status::WT_TYPECHANGE) => {
                    Status::Changed("typechange".to_owned())
                }
                s if s.contains(git2::Status::WT_NEW) => {
                    Status::Untracked
                }
                _ => continue,
            };

            // used when comparing the two files, but I think we can just use the
            // entry path in this scenario no?
            // let old_path = entry.head_to_index().unwrap().old_file().path();
            // let new_path = entry.head_to_index().unwrap().new_file().path();

            if to_ignore
                .iter()
                .any(|f| entry.path().unwrap().ends_with(f))
            {
                continue;
            }

            let path = entry.path().unwrap().to_owned();

            self.file.insert(path, status);
        }

        Ok(())
    }
}
