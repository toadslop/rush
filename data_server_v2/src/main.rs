use std::{io, net::TcpListener};

use rush_data_server::run;

#[actix_web::main]
async fn main() -> io::Result<()> {
    run(TcpListener::bind("127.0.0.1:8080")?).await
}
