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
