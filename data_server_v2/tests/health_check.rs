// use crate::spawn_app;

use crate::util::spawn_app;

mod util;

#[actix_web::test]
async fn health_check_works() {
    let (address, _) = spawn_app().await.expect("Failed to spawn app.");

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
