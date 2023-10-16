use crate::util::spawn_app;
use fake::faker::internet::en::{FreeEmailProvider, Password, SafeEmail};
use fake::Fake;
use rush_data_server::model::account::Account;
use rush_data_server::model::account::CreateAccountDto;
mod util;

#[actix_web::test]
async fn create_account_returns_200_for_valid_input() {
    let (address, db) = spawn_app().await.expect("Failed to spawn app.");
    let client = reqwest::Client::new();

    let email: String = SafeEmail().fake();
    let password = Password(8..16).fake();

    let body = CreateAccountDto {
        email: email.clone(),
        password,
    };

    let response = client
        .post(format!("{address}/account"))
        .header("Content-Type", "application/json")
        .json::<CreateAccountDto>(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    db.use_ns("root").use_db("root").await.unwrap();

    let result: Option<Account> = db.select(("account", &email)).await.unwrap();

    let account = result.unwrap();

    assert_eq!(200, response.status().as_u16());
    assert_eq!(&email, account.email.clone().unwrap().as_ref());
}

#[actix_web::test]
async fn create_account_returns_400_for_invalid_input() {
    let (address, ..) = spawn_app().await.expect("Failed to spawn app.");
    let client = reqwest::Client::new();
    let test_cases = [
        (
            CreateAccountDto {
                email: "".into(),
                password: Password(8..16).fake(),
            },
            "empty email",
        ),
        (
            CreateAccountDto {
                email: SafeEmail().fake(),
                password: "".into(),
            },
            "empty password",
        ),
        (
            CreateAccountDto {
                email: format!(
                    "{}@{}",
                    "a".repeat(321),
                    FreeEmailProvider().fake::<String>()
                ),
                password: Password(8..16).fake(),
            },
            "email over 320 chars",
        ),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(format!("{address}/account"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 BadRequest when the payload was {}.",
            description
        );
    }
}
