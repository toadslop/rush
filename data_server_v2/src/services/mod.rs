use actix_web::{web, HttpResponse};
use futures_util::join;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use surrealdb::{engine::any::Any, Surreal};

pub mod instance;
pub mod root;
mod util;

#[tracing::instrument(name = "Health check")]
pub async fn health_check(
    db: web::Data<Surreal<Any>>,
    mailer: web::Data<AsyncSmtpTransport<Tokio1Executor>>,
) -> HttpResponse {
    tracing::debug!("Health check requested");
    let (db_health, mailer_health) = join!(async { db.health().await }, mailer.test_connection());

    match (db_health, mailer_health) {
        (Ok(_), Ok(_)) => HttpResponse::Ok().finish(),
        (Ok(_), Err(e)) => {
            tracing::error!("Smtp Server connection is unhealthy: {e}");
            HttpResponse::InternalServerError().body("Failed to connect to smtp server")
        }
        (Err(e), Ok(_)) => {
            tracing::error!("Database connection is unhealthy: {e}");
            HttpResponse::InternalServerError().body("Failed to connect to database server")
        }
        (Err(db_e), Err(smtp_e)) => {
            tracing::error!("Smtp Server connection is unhealthy: {smtp_e}");
            tracing::error!("Database connection is unhealthy: {db_e}");
            HttpResponse::InternalServerError()
                .body("Failed to connect to database server and to mail server")
        }
    }
}
