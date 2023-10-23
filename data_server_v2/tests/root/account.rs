use crate::{
    root::{account::helpers::extract_message_body, fakes::DummyAccountDto},
    util::{spawn_app, MailMessageFormat, TestSettings},
};
use fake::{
    faker::{
        company::en::CompanyName,
        internet::en::{FreeEmailProvider, Password, SafeEmail},
    },
    Fake, Faker,
};
use rush_data_server::model::account::{Account, CreateAccountDto};

use self::helpers::{extract_confirmation_email, extract_confirmation_link, post_dummy_account};

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

#[actix_web::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let test_app = spawn_app(TestSettings { spawn_smtp: true }).await.unwrap();
    let response = test_app.confirm_account(None).await;
    assert_eq!(response.status().as_u16(), 400);
}

#[actix_web::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let mut test_app = spawn_app(TestSettings { spawn_smtp: true }).await.unwrap();

    post_dummy_account(&test_app).await;
    let message = extract_confirmation_email(&mut test_app).await;
    let body = extract_message_body(&mut test_app, message.id, MailMessageFormat::Html).await;

    let raw_link = extract_confirmation_link(&body);
    let link = reqwest::Url::parse(&raw_link).unwrap();

    assert_eq!(
        link.host_str().unwrap(),
        test_app.app_address.host_str().unwrap()
    );
    println!("HERE {}", link);
    dbg!(link.to_string());
    let response = reqwest::get(link).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}

mod helpers {
    use fake::{Fake, Faker};
    use rush_data_server::model::account::Account;

    use crate::{
        root::fakes::DummyAccountDto,
        util::{MailMessageFormat, MailtutanJsonMail, TestApp},
    };

    pub async fn extract_message_body(
        test_app: &mut TestApp,
        id: usize,
        format: MailMessageFormat,
    ) -> String {
        test_app
            .smtp_client
            .as_mut()
            .expect("should have client")
            .get_message(id, format)
            .await
    }

    pub fn extract_confirmation_link(s: &str) -> String {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);

        links[0].as_str().to_owned()
    }

    pub async fn post_dummy_account(test_app: &TestApp) -> Account {
        let body: DummyAccountDto = Faker.fake();

        test_app
            .post_account(&body)
            .await
            .json()
            .await
            .expect("Failed to deserialize account from response.")
    }

    pub async fn extract_confirmation_email(test_app: &mut TestApp) -> MailtutanJsonMail {
        let messages = test_app
            .smtp_client
            .as_mut()
            .expect("This test requires an smtp client. Set 'spawn_smtp' to true in TestSettings")
            .get_messages()
            .await;

        messages
            .get(0)
            .expect("There should have been a first email message")
            .clone()
    }
}
