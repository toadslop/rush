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

    let raw_link = extract_confirmation_link(&body);
    let link = reqwest::Url::parse(&raw_link).unwrap();

    assert_eq!(
        link.host_str().unwrap(),
        test_app.app_address.host_str().unwrap()
    );

    let response = reqwest::get(link).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}
