use rush_data_server::{
    configuration::{get_configuration, ApplicationSettings, Settings},
    database::init_db,
    run,
    telemetry::init_telemetry,
};
use std::{io, net::TcpListener};

#[actix_web::main]
async fn main() -> io::Result<()> {
    init_telemetry()?;
    let Settings {
        database,
        application,
    } = get_configuration().expect("Failed to read configuration.");

    let ApplicationSettings { host, port } = application;
    let address = format!("{host}:{port}");

    let db = init_db(database).await.expect("Could not initialize db");

    let listener = TcpListener::bind(address)?;
    run(listener, db).await
}
