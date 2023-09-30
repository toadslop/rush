use actix_web::HttpResponse;

pub mod instance;

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
