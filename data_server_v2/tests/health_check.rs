use actix_web::rt::spawn;
use std::{io, net::TcpListener};

#[actix_web::test]
async fn health_check_works() {
    let address = spawn_app().await.expect("Failed to spawn our app.");

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
// Launch our application in the background ~somehow~
async fn spawn_app() -> io::Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = rush_data_server::run(listener);
    spawn(server);

    Ok(format!("http://127.0.0.1:{}", port))
}
