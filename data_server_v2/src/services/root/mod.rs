use crate::guards::instance_filter::instance_filter;

use self::instance::create_instance;
use actix_web::{
    guard::{self, fn_guard},
    web,
};

mod instance;

pub fn root_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/instance")
            .guard(guard::Not(fn_guard(instance_filter)))
            .route(web::post().to(create_instance)),
    );
}
