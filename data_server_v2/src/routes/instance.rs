use actix_web::{web, HttpResponse};

use crate::model::instance::Instance;

pub async fn create_instance(instance: web::Json<Instance>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
