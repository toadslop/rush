use crate::db::initialize;
use actix_web::{guard, web, App, HttpResponse, HttpServer, Responder};
use api::api;
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
    initialize(&DB, db::AppType::Root).await.unwrap();

    let hosts = ["host1", "host2"];

    // HttpServer::new(move || {
    //     hosts.iter().fold(App::new(), |app, host| {
    //         app.service(web::scope("/").guard(guard::Host(host)).configure(api))
    //     })
    // })
    // .bind(("127.0.0.1", 8080))?
    // .run()
    // .await

    HttpServer::new(|| App::new().configure(api))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    HttpServer::new(|| {
        App::new().service(
            // prefixes all resources and routes attached to it...
            web::scope("/app")
                // ...so this handles requests for `GET /app/index.html`
                .route("/index.html", web::get().to(manual_hello)),
        )
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await?;

    Ok(())
}
async fn manual_hello() -> impl Responder {
    println!("HERE");
    HttpResponse::Ok().body("Hey there!")
}
