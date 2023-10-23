use crate::util::{spawn_app, TestApp, TestSettings};

const HEALTH_CHECK_ENDPOINT: &str = "/health_check";

#[actix_web::test]
async fn health_check_works() {
    let TestApp { app_address, .. } = spawn_app(TestSettings { spawn_smtp: false })
        .await
        .expect("Failed to spawn app.");

    let client = reqwest::Client::new();

    let response = client
        .get(app_address.join(HEALTH_CHECK_ENDPOINT).unwrap())
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
