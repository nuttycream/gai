use crate::{
    args::GlobalArgs,
    git::{Diffs, GitRepo},
    settings::{Settings, load},
};

pub struct State {
    pub settings: Settings,
    pub git: GitRepo,

    /// diff database
    /// we'll use this to compare difflines
    /// over the next commits
    /// during hunk staging
    /// otherwise, diffs
    /// in these will get removed
    /// as we apply them
    pub diffs: Diffs,
}

impl State {
    pub fn new(
        overrides: Option<&[String]>,
        global_args: &GlobalArgs,
    ) -> anyhow::Result<Self> {
        let mut settings = load::load(overrides)?;

        if let Some(provider) = global_args.provider {
            settings.provider = provider;
        }

        if let Some(ref hint) = global_args.hint {
            settings.prompt.hint = Some(hint.to_owned());
        }

        let git = GitRepo::open(None)?;
        let diffs = Diffs::default();

        Ok(Self {
            settings,
            git,
            diffs,
        })
    }
}
