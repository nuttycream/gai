use crate::{
    args::{GlobalArgs, RewordArgs},
    git::{
        log::{get_log, get_logs},
        status::is_workdir_clean,
    },
    providers::provider::ProviderKind,
    schema::SchemaSettings,
    state::State,
};

pub fn run(
    args: &RewordArgs,
    global: &GlobalArgs,
) -> anyhow::Result<()> {
    let state = State::new(
        global
            .config
            .as_deref(),
        global,
    )?;

    if !is_workdir_clean(&state.git.repo)? {
        return Err(anyhow::anyhow!(
            "Workdir is NOT clean, please save your changes"
        ));
    }

    // single commit
    if let Some(ref commit_hash) = args.commit {
        let log = get_log(&state.git, &commit_hash)?;
    }

    let schema_settings = if matches!(
        state
            .settings
            .provider,
        ProviderKind::OpenAI
    ) {
        SchemaSettings::default()
            .additional_properties(false)
            .allow_min_max_ints(true)
    } else {
        SchemaSettings::default().allow_min_max_ints(true)
    };

    Ok(())
}
