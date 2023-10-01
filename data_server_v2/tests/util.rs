use actix_web::rt::spawn;
use once_cell::sync::Lazy;
use rush_data_server::{
    configuration::{get_configuration, Settings},
    database::init_db,
    telemetry::init_telemetry,
};
use std::{env, io, net::TcpListener};
use surrealdb::{engine::any::Any, Surreal};

pub async fn spawn_app() -> io::Result<(String, Surreal<Any>)> {
    env::set_var("APP_ENVIRONMENT", "test");
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let Settings { database, .. } = get_configuration().expect("Failed to read configuration.");
    let db = init_db(database).await.expect("Could not initialize db");
    let server = rush_data_server::run(listener, db.clone());
    spawn(server);

    Ok((format!("http://127.0.0.1:{}", port), db))
}

static TRACING: Lazy<io::Result<()>> = Lazy::new(|| {
    init_telemetry()?;

    Ok(())
});
