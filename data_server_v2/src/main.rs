use rush_data_server::{
    configuration::{get_configuration, Settings},
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
        application_port,
    } = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", application_port);

    init_db(database).await.expect("Could not initialize db");

    let listener = TcpListener::bind(address)?;
    run(listener).await
}
