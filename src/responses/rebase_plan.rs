use crate::schema::rebase_plan::PlanOperationSchema;

/// extract rebase_plan from
/// response
pub fn parse_from_rebase_plan_schema(
    value: serde_json::Value
) -> anyhow::Result<Vec<PlanOperationSchema>> {
    todo!()
}
