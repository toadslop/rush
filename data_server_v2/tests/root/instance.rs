use crate::{
    root::fakes::{DummyAccountDto, DummyCreateInstanceDto},
    util::{spawn_app, TestSettings},
};
use fake::{
    faker::{
        company::{en::BsAdj, en::BsNoun},
        internet::en::SafeEmail,
    },
    Fake, Faker,
};
use rush_data_server::model::{
    account::{Account, CreateAccountDb},
    instance::{CreateInstanceDto, Instance},
    Table,
};
use surrealdb::opt::RecordId;

#[actix_web::test]
async fn create_instance_returns_200_for_valid_input() {
    let test_app = spawn_app(TestSettings { spawn_smtp: false })
        .await
        .expect("Failed to spawn app.");

    test_app
        .db
        .use_ns("root")
        .use_db("root")
        .await
        .expect("Failed to connect to root ns and root db");

    let _dummy_account: DummyAccountDto = Faker.fake();
    let dummy_account: CreateAccountDb = (*_dummy_account).clone().into();

    test_app
        .db
        .create::<Option<Account>>((Account::name(), _dummy_account.email.clone()))
        .content(&dummy_account)
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to create test account");

    let instance_name = "my-instance";

    let body = CreateInstanceDto {
        name: instance_name.into(),
        account_id: _dummy_account.email.clone(),
    };

    let response = test_app.post_instance(&body).await;

    assert_eq!(200, response.status().as_u16());
    let body = &response.text().await.unwrap();
    dbg!(body);

    let instance: Option<Instance> = test_app
        .db
        .select((Instance::name(), instance_name))
        .await
        .expect("Failed to find the created instance in the database");

    dbg!(&instance);

    let name = instance
        .expect("An instance should have been created")
        .name
        .expect("Instance should have a name");

    assert_eq!(instance_name, name);
    dbg!(&_dummy_account);
    let mut result = test_app
        .db
        .query("SELECT instances[WHERE $instance_id] FROM $account_id")
        .bind((
            "instance_id",
            RecordId::from((Instance::name(), instance_name)).to_string(),
        ))
        .bind((
            "account_id",
            RecordId::from((Account::name(), _dummy_account.email.clone().as_ref())),
        ))
        .await
        .map_err(|e| e.to_string())
        .unwrap();

    let account: Option<Account> = result
        .take(0)
        .map_err(|e| e.to_string())
        .expect("Encountered an error when trying to take the account");

    let account = account.expect("Tried to unwrap the account but got none");
    dbg!(&account);
    let instances = account
        .instances
        .expect("Should have found a vec of instances but it was None");
    let instance_id = instances
        .get(0)
        .expect("Should have been an instance at index 0 but got none");

    assert_eq!(
        *instance_id,
        RecordId::from((Instance::name(), instance_name))
    )
}

#[actix_web::test]
async fn create_instance_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app(TestSettings { spawn_smtp: false })
        .await
        .expect("Failed to spawn app.");

    let test_cases = [
        (
            CreateInstanceDto {
                name: format!("{}_{}", BsAdj().fake::<String>(), BsNoun().fake::<String>()),
                account_id: "".into(),
            },
            "no account id",
        ),
        (
            CreateInstanceDto {
                name: "".into(),
                account_id: SafeEmail().fake(),
            },
            "no instance name",
        ),
    ];

    for (body, error_message) in test_cases {
        let response = test_app.post_instance(&body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[actix_web::test]
async fn trying_to_create_instance_when_not_logged_in_returns_403() {
    // Arrange
    let test_app = spawn_app(TestSettings { spawn_smtp: false })
        .await
        .expect("Failed to spawn app.");

    let _dummy_account: DummyAccountDto = Faker.fake();
    let dummy_account: CreateAccountDb = (*_dummy_account).clone().into();

    test_app
        .db
        .create::<Option<Account>>((Account::name(), _dummy_account.email.clone()))
        .content(&dummy_account)
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to create test account");

    let mut dummy_instance: DummyCreateInstanceDto = Faker.fake();
    dummy_instance.account_id = _dummy_account.email.clone();

    // Act
    let resp = test_app.post_instance(&dummy_instance).await;

    // Assert
    assert_eq!(
        401,
        resp.status().as_u16(),
        "The API did not fail with 403 Not Authorized when the requester was not logged in"
    );
}
