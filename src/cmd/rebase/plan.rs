use serde_json::Value;

use crate::{
    git::Diffs,
    providers::extract_from_provider,
    requests::rebase_plan::create_rebase_plan_request,
    responses::rebase_plan::parse_from_rebase_plan_schema,
    schema::{
        SchemaSettings,
        rebase_plan::{
            PlanOperationSchema, create_rebase_plan_schema,
        },
    },
    settings::Settings,
};

/// a gai rebase --plan will operate significantly
/// different than the regular gai rebase.
/// one: it will not generate commits, instead
/// it will generate a list of RebaseOperationTypes'
/// two: since it generates rebase operations, applying these will
/// HANDLE ALOT differently, in terms of what can be rejected,
/// as well as the flow within git itself
/// WTF
pub(super) fn gen_plan(
    settings: &Settings,
    diffs: &Diffs,
    logs: &[String],
    schema_settings: &SchemaSettings,
) -> anyhow::Result<Option<Vec<PlanOperationSchema>>> {
    let request = create_rebase_plan_request(
        settings,
        logs,
        &diffs.to_string(),
    );

    let schema = create_rebase_plan_schema(
        schema_settings.to_owned(),
        logs.len(),
        false,
    )?;

    loop {
        let response: Value = match extract_from_provider(
            &settings.provider,
            request.to_owned(),
            schema.to_owned(),
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error from the provider:\n{e}");
                break;
            }
        };

        let _raw_ops = parse_from_rebase_plan_schema(response)?;
        //println!("{:#?}", raw_ops);

        return Ok(None);
        // if let Some(opt) = print_rebase_plan(&raw_ops, false)? {
        //     if opt == 0 {
        //         println!("Applying");
        //         return Ok(Some(raw_ops));
        //     } else if opt == 1 {
        //         println!("Regenerating");
        //         continue;
        //     }
        // } else {
        //     println!("Exiting");
        //     return Ok(None);
        // }
    }

    Ok(None)
}
