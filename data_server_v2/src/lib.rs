use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use middleware::virtual_hosting::VirtualHostProcessor;
use routes::{health_check, instance::create_instance};
use std::{io, net::TcpListener};
use surrealdb::{engine::any::Any, Surreal};
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod database;
mod middleware;
pub mod model;
mod routes;
pub mod telemetry;

pub async fn run(listener: TcpListener, db: Surreal<Any>) -> io::Result<()> {
    // TODO: create instance guard to handle directing to instance handling or main admin instance
    // TODO: set up proper tracing logs for existing endpoints and middleware
    HttpServer::new(move || {
        App::new()
            .wrap(VirtualHostProcessor)
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/instance", web::post().to(create_instance))
            .app_data(Data::new(db.clone()))
    })
    .listen(listener)?
    .run()
    .await
}
