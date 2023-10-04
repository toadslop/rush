use actix_web::guard::GuardContext;

use crate::model::instance::InstanceName;

#[tracing::instrument]
pub fn instance_filter(ctx: &GuardContext) -> bool {
    tracing::debug!("Hit instance filter");
    let result = ctx.req_data().get::<InstanceName>().is_some();
    tracing::debug!("Instance name found: {result}");
    result
}
