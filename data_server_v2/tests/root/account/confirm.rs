use rush_data_server::model::account::Account;

use crate::{
    root::account::helpers::{
        extract_confirmation_email, extract_confirmation_link, extract_message_body,
        post_dummy_account,
    },
    util::{spawn_app, MailMessageFormat, TestSettings},
};

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
    let link = extract_confirmation_link(&body);
    dbg!(&link);
    assert_eq!(
        link.host_str().unwrap(),
        test_app.app_address.host_str().unwrap()
    );

    let response = reqwest::get(link).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}

#[actix_web::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // arrange
    let mut test_app = spawn_app(TestSettings { spawn_smtp: true }).await.unwrap();
    let account = post_dummy_account(&test_app).await;
    let message = extract_confirmation_email(&mut test_app).await;
    let body = extract_message_body(&mut test_app, message.id, MailMessageFormat::Html).await;
    let link = extract_confirmation_link(&body);

    // Act
    reqwest::get(link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Assert
    let saved: Option<Account> = test_app.db.select(account.id.unwrap()).await.unwrap();
    let saved = saved.unwrap();
    dbg!(&saved);
    assert_eq!(saved.email.unwrap(), account.email.unwrap());
    assert_eq!(saved.name.unwrap(), account.name.unwrap());
    assert!(saved.confirmed.unwrap());
}
