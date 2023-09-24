use std::{io, net::TcpListener};

use actix_web::{web, App, HttpServer};
use routes::health_check;
use telemetry::init_telemetry;
use tracing_actix_web::TracingLogger;

mod routes;
mod telemetry;

pub async fn run(listener: TcpListener) -> io::Result<()> {
    init_telemetry();

    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run()
    .await
}
