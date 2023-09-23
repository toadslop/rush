use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, Responder};
use api::api;
use db::admin::initialize_admin;
use env_logger::Env;
// use log::{debug, error, info, log_enabled};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};
use tracing::{event, info, span, Level};
use tracing_actix_web::TracingLogger;

mod api;
mod db;
mod models;
mod pipe;

static DB: Surreal<Client> = Surreal::init();

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // env_logger::init_from_env(Env::default().default_filter_or("info"));
    let span = span!(Level::INFO, "startup").entered();

    info!("Initializing Rush data server");

    info!("Connecting to database...");

    // event!(Level::INFO, "Rush data server initializting...");
    // event!(Level::INFO, "Connecting to database...");
    DB.connect::<Ws>("localhost:8000")
        .await
        .expect("Failed to connect to the db"); // TODO: Load from env
                                                // info!("Database connect successful.");
    event!(Level::INFO, "Database connection successful");

    initialize_admin(&DB, "root", "root")
        .await
        .expect("Should have been able to initialize");

    HttpServer::new(|| {
        App::new()
            // .wrap(Logger::default())
            // .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(TracingLogger::default())
            .configure(api)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
async fn manual_hello() -> impl Responder {
    println!("HERE");
    HttpResponse::Ok().body("Hey there!")
}
