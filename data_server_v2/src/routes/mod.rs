use actix_web::HttpResponse;

pub mod instance;

#[tracing::instrument(name = "Health check requested")]
pub async fn health_check() -> HttpResponse {
    tracing::debug!("Health check requested");
    HttpResponse::Ok().finish()
}
