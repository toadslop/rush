use super::helpers::post_and_confirm_dummy_account;
use crate::util::{spawn_app, TestSettings};
use rush_data_server::model::account::AccountSignin;
use surrealdb::opt::auth::Jwt;

#[actix_web::test]
async fn user_receives_jwt_after_attempting_signin_with_valid_credentials() {
    let mut test_app = spawn_app(TestSettings { spawn_smtp: true }).await.unwrap();
    let account = post_and_confirm_dummy_account(&mut test_app).await;

    let res = test_app
        .signin_account(&AccountSignin {
            email: account.email.as_ref().unwrap().clone(),
            password: account.password.as_ref().unwrap().to_owned(),
        })
        .await;

    res.json::<Jwt>()
        .await
        .expect("Failed to get valid JWT from signin endpoint");
}
