use actix_web::{web, App, HttpServer};
use routes::{health_check, instance::create_instance};
use std::{io, net::TcpListener};
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod database;
mod model;
mod routes;
pub mod telemetry;

pub async fn run(listener: TcpListener) -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/instance", web::post().to(create_instance))
    })
    .listen(listener)?
    .run()
    .await
}
