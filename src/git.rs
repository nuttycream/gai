use color_eyre::Result;
use std::{collections::HashMap, path::Path};

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

pub enum Status {
    /// change includes
    /// modified, deleted,
    /// typechanged, renamed
    Changed,

    /// files that haven't been
    /// added Status::WT_NEW
    Untracked,
}

impl GitState {
    /// this could fail on an unitialized directory
    /// for now, im not gonna handle those and we
    /// just straight up panic if we failed to open
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        let mut options = StatusOptions::new();

        options.include_untracked(true);

        Ok(GitState {
            repo,
            options,
            file: HashMap::new(),
        })
    }

    pub fn get_status(&mut self) -> Result<()> {
        let statuses = self.repo.statuses(Some(&mut self.options))?;

        for entry in statuses.iter() {}

        Ok(())
    }
}
