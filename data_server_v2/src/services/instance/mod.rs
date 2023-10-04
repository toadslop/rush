use actix_web::{guard, web, HttpMessage, HttpRequest, HttpResponse};

use crate::{guards::instance_filter::instance_filter, model::instance::InstanceName};

pub fn instance_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .guard(guard::fn_guard(instance_filter))
            .route(web::get().to(instance_name)),
    );
}

#[tracing::instrument]
pub async fn instance_name(req: HttpRequest) -> HttpResponse {
    tracing::trace!("Reached create_instance route handler");
    if let Some(instance_name) = req.extensions().get::<InstanceName>() {
        HttpResponse::Ok().body::<String>(instance_name.to_string())
    } else {
        HttpResponse::Ok().finish()
    }
}
