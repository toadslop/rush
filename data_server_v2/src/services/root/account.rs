use crate::model::account::Account;
use crate::model::account::CreateAccount;
use actix_web::web;
use actix_web::HttpResponse;
use surrealdb::{engine::any::Any, Surreal};

#[tracing::instrument]
pub async fn create_route(
    instance: web::Json<CreateAccount>,
    db: web::Data<Surreal<Any>>,
) -> HttpResponse {
    tracing::trace!("Reached create_route route handler");
    let resp = match create_account_db(instance, db).await {
        Ok(instance) => HttpResponse::Ok().json(instance),
        Err(_) => HttpResponse::InternalServerError().finish(),
    };
    tracing::trace!("Handler exited");
    resp
}

#[tracing::instrument(skip(db))]
async fn create_account_db(
    account: web::Json<CreateAccount>,
    db: web::Data<Surreal<Any>>,
) -> Result<Vec<Account>, surrealdb::Error> {
    tracing::info!("Attempting to saving new instance to the db");
    let account = db
        .create::<Vec<Account>>("account")
        .content(Account::from(account.into_inner()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to persist instance to db: {:?}", e);
            e
        })?;
    tracing::info!("Success");
    Ok(account)
}
