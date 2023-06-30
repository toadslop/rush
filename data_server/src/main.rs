use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use api::api;
use db::instance::{initialize_instance, AppType};
// use shared::anyhow::Ok;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};

mod api;
mod db;
mod models;
mod pipe;

static DB: Surreal<Client> = Surreal::init();

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    DB.connect::<Ws>("localhost:8000").await.unwrap();
    initialize_instance(&DB, AppType::Root).await.unwrap();

    HttpServer::new(|| App::new().configure(api))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
async fn manual_hello() -> impl Responder {
    println!("HERE");
    HttpResponse::Ok().body("Hey there!")
}
