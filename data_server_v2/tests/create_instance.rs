use crate::util::spawn_app;

mod util;

#[actix_web::test]
async fn create_instance_returns_200_for_valid_input() {
    let address = spawn_app().await.expect("Failed to spawn app.");

    let client = reqwest::Client::new();

    let body = r#"{ "name": "my-instance" }"#;

    let response = client
        .post(format!("{address}/instance"))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app().await.expect("Failed to spawn app.");
    let client = reqwest::Client::new();
    let test_cases = [
        ("", "no data"),
        (r#"{ "notName": "bobby" }"#, "missing the instances name"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/instance", &address))
            .header("Content-Type", "application/json")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
