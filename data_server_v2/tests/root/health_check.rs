use crate::util::{spawn_app, TestApp};

#[actix_web::test]
async fn health_check_works() {
    let TestApp { app_address, .. } = spawn_app().await.expect("Failed to spawn app.");

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{app_address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
