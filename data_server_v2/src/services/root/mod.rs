use self::{
    account::{confirm::confirm, signin::sign_in, signup::create_account},
    instance::create_instance,
};
use crate::guards::instance_filter::instance_filter;
use actix_web::{
    guard::{self, fn_guard},
    web,
};
mod account;
mod instance;

/// The root service handles the metadata required to define instances
pub fn root_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/instance")
            .guard(guard::Not(fn_guard(instance_filter)))
            .route(web::post().to(create_instance)),
    )
    .service(
        web::scope("/account")
            .guard(guard::Not(fn_guard(instance_filter)))
            .route("", web::post().to(create_account))
            .route("/confirm", web::get().to(confirm))
            .route("/signin", web::post().to(sign_in)),
    );
}
