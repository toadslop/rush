use actix_web::{web, HttpResponse};

use crate::{database::DB, model::instance::Instance};

pub async fn create_instance(instance: web::Json<Instance>) -> HttpResponse {
    let instance: Vec<Instance> = DB
        .create("instance")
        .content(instance)
        .await
        .expect("Failed to create the instances");
    HttpResponse::Ok().json(instance)
}
