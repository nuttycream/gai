use serde_json::Value;

use crate::{
    git::Diffs,
    print::{
        loading, print_retry_prompt, rebase_plan::print_rebase_plan,
    },
    providers::extract_from_provider,
    requests::rebase_plan::create_rebase_plan_request,
    responses::rebase_plan::parse_from_rebase_plan_schema,
    schema::{
        SchemaSettings, rebase_plan::create_rebase_plan_schema,
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
pub(super) fn gen_plan(
    settings: &Settings,
    diffs: &Diffs,
    logs: &[String],
    schema_settings: &SchemaSettings,
) -> anyhow::Result<Option<()>> {
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
        let loading =
            loading::Loading::new("Generating Rebase Plan", false)?;

        loading.start();

        let response: Value = match extract_from_provider(
            &settings.provider,
            request.to_owned(),
            schema.to_owned(),
        ) {
            Ok(r) => r,
            Err(e) => {
                let msg = format!(
                    "Gai received an error from the provider:\n{:#}\nRetry?",
                    e
                );

                loading.stop();

                if print_retry_prompt(Some(&msg))? {
                    continue;
                } else {
                    break;
                }
            }
        };

        let raw_ops = parse_from_rebase_plan_schema(response)?;
        //println!("{:#?}", raw_ops);

        loading.stop();

        if let Some(opt) = print_rebase_plan(&raw_ops, false)? {
        } else {
            println!("Exiting");
            return Ok(None);
        }

        // let selected = match print_response_commits(
        //     &raw_ops,
        //     global.compact,
        //     matches!(
        //         state
        //             .settings
        //             .staging_type,
        //         StagingStrategy::Hunks
        //     ),
        //     false,
        // )? {
        //     Some(s) => s,
        //     None => {
        //         println!("Exiting...");
        //         return Ok(());
        //     }
        // };
        //
        // if selected == 0 {
        // } else if selected == 1 {
        //     println!("Regenerating");
        //     continue;
        // } else if selected == 2 {
        //     println!("Exiting");
        //     break;
        // }
    }

    Ok(Some(()))
}
