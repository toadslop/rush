use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use configuration::app::ApplicationSettings;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use middleware::virtual_hosting::VirtualHostProcessor;
use services::{health_check, instance::instance_service, root::root_service};
use std::{fmt::Display, io, net::TcpListener};
use surrealdb::{engine::any::Any, Surreal};
use tracing_actix_web::TracingLogger;

pub mod configuration;
pub mod database;
mod guards;
pub mod mailer;
mod middleware;
pub mod model;
mod services;
pub mod startup;
pub mod telemetry;
pub mod util;

pub async fn run(
    listener: TcpListener,
    db: Surreal<Any>,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    settings: ApplicationSettings,
) -> io::Result<Server> {
    // TODO: create instance guard to handle directing to instance handling or main admin instance
    // TODO: set up proper tracing logs for existing endpoints and middleware
    let app_address = listener.local_addr().unwrap();
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(VirtualHostProcessor)
            .wrap(TracingLogger::default())
            .configure(root_service)
            .configure(instance_service)
            .route("/health_check", web::get().to(health_check))
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(mailer.clone()))
            .app_data(Data::new(settings.clone()))
            .app_data(Data::new(AppAddress(app_address.to_string())))
    })
    .listen(listener)?
    .run())
}

pub struct AppAddress(pub String);
impl Display for AppAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
