use crate::{
    git::Diffs, requests::rebase_plan::create_rebase_plan_request,
    schema::SchemaSettings, settings::Settings,
};

/// a gai rebase --plan will operate significantly
/// different than the regular gai rebase.
/// one: it will not generate commits, instead
/// it will generate a list of RebaseOperationTypes'
/// two: since it generates rebase operations, applying these will
/// HANDLE ALOT differently, in terms of what can be rejected,
/// as well as the flow within git itself
pub(super) fn gen_plan(
    settings: &Settings,
    diffs: &Diffs,
    logs: &[String],
    schema_settings: &SchemaSettings,
) -> anyhow::Result<()> {
    create_rebase_plan_request(settings, logs, &diffs.to_string());
    Ok(())
}
