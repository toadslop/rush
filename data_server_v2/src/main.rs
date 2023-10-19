use rush_data_server::{
    configuration::{app::ApplicationSettings, get_configuration, Settings},
    database::init_db,
    mailer::init_mailer,
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
        mail,
    } = get_configuration().expect("Failed to read configuration.");

    let ApplicationSettings { host, port } = application;
    let address = format!("{host}:{port}");

    let db = init_db(database).await.expect("Could not initialize db");
    let mailer = init_mailer(mail);

    let listener = TcpListener::bind(address)?;
    run(listener, db, mailer).await
}
