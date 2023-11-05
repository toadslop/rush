use fake::{Fake, Faker};
use rush_data_server::model::account::{Account, AccountSignin};
use rush_data_server::model::email_address::EmailAddress;
use rush_data_server::model::Table;
use surrealdb::opt::auth::Jwt;
use surrealdb::opt::RecordId;
use surrealdb::sql::Id;

use crate::{
    root::fakes::DummyAccountDto,
    util::{spawn_app, TestSettings},
};

#[actix_web::test]
async fn user_receives_jwt_after_attempting_signin_with_valid_credentials() {
    let test_app = spawn_app(TestSettings { spawn_smtp: false }).await.unwrap();

    let _dummy_account: DummyAccountDto = Faker.fake();
    // let _dummy_account: CreateAccountDb = (*_dummy_account).clone().into();
    let dummy_account = Account {
        id: Some(RecordId {
            tb: Account::name().to_string(),
            id: Id::String(_dummy_account.email.clone()),
        }),
        email: Some(EmailAddress(_dummy_account.email.clone())),
        name: Some(_dummy_account.name.clone()),
        confirmed: Some(true),
        instances: Some(vec![]),
        created_by: None,
        updated_by: None,
        created_at: None,
        updated_at: None,
        password: Some(_dummy_account.password.clone()),
    };

    let account = test_app
        .db
        .create::<Option<Account>>((Account::name(), _dummy_account.email.clone()))
        .content(&dummy_account)
        .await
        .map_err(|e| e.to_string())
        .expect("Failed to create test account");

    let account = account.expect("Failed to create a test account");

    dbg!(account);

    let res = test_app
        .signin_account(&AccountSignin {
            email: EmailAddress(_dummy_account.email.to_owned()),
            password: _dummy_account.password.to_owned(),
        })
        .await;

    let jwt = res
        .json::<Jwt>()
        .await
        .expect("Failed to get valid JWT from signing endpoint");

    println!("{}", jwt.into_insecure_token());
}
