use crate::schema::rebase_plan::{
    PlanOperationSchema, RebasePlanResponse,
};

/// extract rebase_plan from
/// response returns a list of
/// operations
/// TODO: impl validation
pub fn parse_from_rebase_plan_schema(
    value: serde_json::Value
) -> anyhow::Result<Vec<PlanOperationSchema>> {
    let plan_resp =
        serde_json::from_value::<RebasePlanResponse>(value)?;

    Ok(plan_resp.operations)
}
