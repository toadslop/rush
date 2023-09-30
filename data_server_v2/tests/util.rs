use actix_web::rt::spawn;
use once_cell::sync::Lazy;
use rush_data_server::telemetry::init_telemetry;
use std::{io, net::TcpListener};

// pub struct TestApp {
//     pub address: String,
// }

pub async fn spawn_app() -> io::Result<String> {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = rush_data_server::run(listener);
    spawn(server);

    Ok(format!("http://127.0.0.1:{}", port))
}

static TRACING: Lazy<io::Result<()>> = Lazy::new(|| {
    init_telemetry()?;

    Ok(())
});
