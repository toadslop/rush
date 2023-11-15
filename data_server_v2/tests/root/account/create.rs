use crate::{
    root::{
        account::helpers::{self, extract_confirmation_link, extract_message_body},
        fakes::DummyAccountDto,
    },
    util::{spawn_app, MailMessageFormat, TestSettings},
};
use fake::{
    faker::{
        company::en::CompanyName,
        internet::en::{FreeEmailProvider, Password},
    },
    Fake, Faker,
};
use rush_data_server::model::account::{Account, CreateAccountDto};

#[actix_web::test]
async fn create_account_returns_200_for_valid_input() {
    let test_app = spawn_app(TestSettings { spawn_smtp: true })
        .await
        .expect("Failed to spawn app.");

    let body: DummyAccountDto = Faker.fake();
    let response = test_app.post_account(&body).await;

    test_app.db.use_ns("root").use_db("root").await.unwrap();

    let result: Option<Account> = test_app.db.select(("account", &body.email)).await.unwrap();

    let account = result.unwrap();

    assert_eq!(200, response.status().as_u16());
    assert_eq!(&body.email, account.email.clone().unwrap().as_ref());
}

#[actix_web::test]
async fn create_account_persists_the_new_user() {
    let test_app = spawn_app(TestSettings { spawn_smtp: true })
        .await
        .expect("Failed to spawn app.");

    let body: DummyAccountDto = Faker.fake();
    test_app.post_account(&body).await;

    test_app.db.use_ns("root").use_db("root").await.unwrap();

    let result: Option<Account> = test_app.db.select(("account", &body.email)).await.unwrap();

    let account = result.unwrap();

    assert_eq!(body.email, account.email.clone().unwrap().0);
    assert_eq!(body.name, account.name.clone().unwrap());
    assert!(!account.confirmed.unwrap());
    assert_eq!(account.instances.unwrap().len(), 0);
}

#[actix_web::test]
async fn create_account_returns_400_for_invalid_input() {
    let test_app = spawn_app(TestSettings { spawn_smtp: true })
        .await
        .expect("Failed to spawn app.");

    let test_cases = [
        (
            CreateAccountDto {
                email: "".into(),
                name: CompanyName().fake::<String>(),
                password: Password(8..16).fake(),
            },
            "empty email",
        ),
        // TODO: resolve password validation in light of fact that encryption means we can't validate strength of password
        // at DB layer
        // (
        //     CreateAccountDto {
        //         email: SafeEmail().fake(),
        //         name: CompanyName().fake::<String>(),
        //         password: "".into(),
        //     },
        //     "empty password",
        // ),
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
        let response = test_app.post_account(&body).await;

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
    let mut test_app = spawn_app(TestSettings { spawn_smtp: true })
        .await
        .expect("Failed to spawn app.");

    let response: Account = helpers::post_dummy_account(&test_app).await;

    let account_email = response
        .email
        .expect("The response should have included an email");

    let message = helpers::extract_confirmation_email(&mut test_app).await;

    let email = message
        .recipients
        .get(0)
        .expect("There should have been a recipient but there wasn't");

    assert_eq!(&account_email.0, email)
}

#[actix_web::test]
async fn create_account_send_a_confirmation_email_with_a_link() {
    // Arrange
    let mut test_app = spawn_app(TestSettings { spawn_smtp: true })
        .await
        .expect("Failed to spawn app.");
    helpers::post_dummy_account(&test_app).await;
    let message = helpers::extract_confirmation_email(&mut test_app).await;
    let html = extract_message_body(&mut test_app, message.id, MailMessageFormat::Html).await;

    let plain = extract_message_body(&mut test_app, message.id, MailMessageFormat::Plain).await;

    let html_link = extract_confirmation_link(&html);
    let plain_link = extract_confirmation_link(&plain);

    assert_eq!(html_link, plain_link);
}
