use fake::{
    faker::internet::en::{Password, SafeEmail},
    Fake,
};
use rush_data_server::model::{
    account::{Account, CreateAccountDb, CreateAccountDto},
    instance::{CreateInstanceDto, Instance},
    Table,
};
use surrealdb::opt::RecordId;

use crate::util::spawn_app;

mod util;

#[actix_web::test]
async fn create_instance_returns_200_for_valid_input() {
    let (address, db) = spawn_app().await.expect("Failed to spawn app.");
    let account_email = SafeEmail().fake::<String>();
    let password = Password(8..16).fake();
    let account: CreateAccountDb = CreateAccountDto {
        email: account_email.clone(),
        password,
    }
    .into();
    db.create::<Option<Account>>((Account::name(), &account_email))
        .content(account)
        .await
        .expect("Failed to create dummy account.");
    let client = reqwest::Client::new();

    let instance_name = "my-instance";

    let body = CreateInstanceDto {
        name: instance_name.into(),
        account_id: account_email.clone(),
    };

    let response = client
        .post(format!("{address}/instance"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    db.use_ns("root").use_db("root").await.unwrap();

    let instance: Option<Instance> = db.select((Instance::name(), instance_name)).await.unwrap();
    let name = instance
        .expect("An instance should have been created")
        .name
        .expect("Instance should have a name");

    assert_eq!(instance_name, name);

    let mut result = db
        .query("SELECT instances[WHERE $instance_id] FROM $account_id")
        .bind((
            "instance_id",
            RecordId::from((Instance::name(), instance_name)).to_string(),
        ))
        .bind((
            "account_id",
            RecordId::from((Account::name(), account_email.as_ref())),
        ))
        .await
        .map_err(|e| e.to_string())
        .unwrap();

    let account: Option<Account> = result.take(0).map_err(|e| e.to_string()).unwrap();

    let account = account.unwrap();
    let instances = account.instances.unwrap();
    let instance_id = instances.get(0).unwrap();

    assert_eq!(
        *instance_id,
        RecordId::from((Instance::name(), instance_name))
    )
}

#[actix_web::test]
async fn create_instance_a_400_when_data_is_missing() {
    let (address, _) = spawn_app().await.expect("Failed to spawn app.");
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
