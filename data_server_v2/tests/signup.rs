use crate::util::spawn_app;
use rush_data_server::model::{account::CreateAccount, instance::Instance};
mod util;

#[actix_web::test]
async fn signup_returns_200_for_valid_input() {
    let (address, db) = spawn_app().await.expect("Failed to spawn app.");
    let client = reqwest::Client::new();

    let body = CreateAccount {
        email: "test".into(),    // TODO: use fake
        password: "test".into(), // TODO: use fake
    };

    let response = client
        .post(format!("{address}/instance"))
        .header("Content-Type", "application/json")
        .json::<CreateAccount>(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    db.use_ns("root").use_db("root").await.unwrap();

    let result: Vec<Instance> = db.select("instance").await.unwrap();
    let name = &result.get(0).unwrap().name;

    assert_eq!(200, response.status().as_u16());
    assert_eq!("my-instance", name);
}
