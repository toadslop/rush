use fake::{Fake, Faker};
use reqwest::Response;
use rush_data_server::model::account::{Account, AccountSignin};
use surrealdb::opt::auth::Jwt;

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

pub fn extract_confirmation_link(s: &str) -> reqwest::Url {
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    assert_eq!(links.len(), 1);

    let raw_link = links[0].as_str().to_owned();
    reqwest::Url::parse(&raw_link).unwrap()
}

pub async fn post_dummy_account(test_app: &TestApp) -> Account {
    let account: DummyAccountDto = Faker.fake();

    let res = test_app.post_account(&account).await;

    let acc: Option<Account> = res
        .json()
        .await
        .expect("Failed to deserialize account from response.");
    let mut acc = acc.unwrap();
    acc.password = Some(account.password.clone());
    acc
}

pub async fn confirm_dummy_account(test_app: &mut TestApp) -> Response {
    let message = extract_confirmation_email(test_app).await;
    let body = extract_message_body(test_app, message.id, MailMessageFormat::Html).await;
    let link = extract_confirmation_link(&body);

    reqwest::get(link).await.unwrap()
}

pub async fn post_and_confirm_dummy_account(test_app: &mut TestApp) -> Account {
    let account = post_dummy_account(test_app).await;
    confirm_dummy_account(test_app).await;
    account
}

pub async fn post_account_and_login(test_app: &mut TestApp) -> (Account, Jwt) {
    let account = post_and_confirm_dummy_account(test_app).await;

    let res = test_app
        .signin_account(&AccountSignin {
            email: account.email.clone().unwrap(),
            password: account.password.clone().unwrap(),
        })
        .await;

    let jwt = res
        .json::<Jwt>()
        .await
        .expect("Failed to get valid JWT from signing endpoint");

    (account, jwt)
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
