use std::{io, net::TcpListener};

use rush_data_server::{run, telemetry::init_telemetry};

#[actix_web::main]
async fn main() -> io::Result<()> {
    init_telemetry()?;
    run(TcpListener::bind("127.0.0.1:8080")?).await
}
