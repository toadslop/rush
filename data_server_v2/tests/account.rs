use crate::fakes::DummyAccountDto;
use crate::util::spawn_app;
use fake::{
    faker::{
        company::en::CompanyName,
        internet::en::{FreeEmailProvider, Password, SafeEmail},
    },
    Fake, Faker,
};
use rush_data_server::model::account::{Account, CreateAccountDto};
mod fakes;
mod util;

#[actix_web::test]
async fn create_account_returns_200_for_valid_input() {
    let (address, db, ..) = spawn_app().await.expect("Failed to spawn app.");
    let client = reqwest::Client::new();

    let fake_account: DummyAccountDto = Faker.fake();

    let response = client
        .post(format!("{address}/account"))
        .header("Content-Type", "application/json")
        .json::<CreateAccountDto>(&*fake_account)
        .send()
        .await
        .expect("Failed to execute request.");

    db.use_ns("root").use_db("root").await.unwrap();

    let result: Option<Account> = db.select(("account", &fake_account.email)).await.unwrap();

    let account = result.unwrap();

    assert_eq!(200, response.status().as_u16());
    assert_eq!(&fake_account.email, account.email.clone().unwrap().as_ref());
}

#[actix_web::test]
async fn create_account_returns_400_for_invalid_input() {
    let (address, ..) = spawn_app().await.expect("Failed to spawn app.");
    let client = reqwest::Client::new();
    let test_cases = [
        (
            CreateAccountDto {
                email: "".into(),
                name: CompanyName().fake::<String>(),
                password: Password(8..16).fake(),
            },
            "empty email",
        ),
        (
            CreateAccountDto {
                email: SafeEmail().fake(),
                name: CompanyName().fake::<String>(),
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
                name: CompanyName().fake::<String>(),
                password: Password(8..16).fake(),
            },
            "email over 320 chars",
        ),
        (
            CreateAccountDto {
                email: format!(
                    "{}@{}",
                    "a".repeat(321),
                    FreeEmailProvider().fake::<String>()
                ),
                name: "".into(),
                password: Password(8..16).fake(),
            },
            "empty company name",
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

#[actix_web::test]
async fn create_account_sends_confirmation_email() {
    let (address, .., mail_relay) = spawn_app().await.expect("Failed to spawn app.");
    let client = reqwest::Client::new();

    let fake_account: DummyAccountDto = Faker.fake();

    let response: Account = client
        .post(format!("{address}/account"))
        .header("Content-Type", "application/json")
        .json::<CreateAccountDto>(&*fake_account)
        .send()
        .await
        .expect("Failed to execute request.")
        .json()
        .await
        .expect("Failed to deserialize account from response.");

    let account_email = response
        .email
        .expect("The response should have included an email");

    let messages = mail_relay.get_messages().await;

    let message = messages
        .get(0)
        .expect("There should have been a first email message");

    let email = message
        .recipients
        .get(0)
        .expect("There should have been a recipient but there wasn't");

    assert_eq!(&account_email.0, email)
}
