use crate::util::spawn_app;
use fake::faker::internet::en::{Password, SafeEmail};
use fake::Fake;
use rush_data_server::model::account::Account;
use rush_data_server::model::account::CreateAccount;
mod util;

#[actix_web::test]
async fn signup_returns_200_for_valid_input() {
    let (address, db) = spawn_app().await.expect("Failed to spawn app.");
    let client = reqwest::Client::new();

    let email: String = SafeEmail().fake();
    let password = Password(8..16).fake();

    let body = CreateAccount {
        email: email.clone(),
        password,
    };

    let response = client
        .post(format!("{address}/account"))
        .header("Content-Type", "application/json")
        .json::<CreateAccount>(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    db.use_ns("root").use_db("root").await.unwrap();

    let result: Vec<Account> = db.select("account").await.unwrap();

    let account = result.get(0).unwrap();

    assert_eq!(200, response.status().as_u16());
    assert_eq!(email, account.email.clone().unwrap());
}
