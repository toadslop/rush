use actix_web::{web, HttpResponse};
use surrealdb::{engine::any::Any, Surreal};

use crate::model::instance::Instance;

pub async fn create_instance(
    instance: web::Json<Instance>,
    db: web::Data<Surreal<Any>>,
) -> HttpResponse {
    let instance: Vec<Instance> = db
        .create("instance")
        .content(instance)
        .await
        .expect("Failed to create the instances");
    HttpResponse::Ok().json(instance)
}
