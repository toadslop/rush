use crate::model::account::Account;
use crate::model::account::CreateAccountDb;
use crate::model::account::CreateAccountDto;
use crate::model::CreateTable;
use actix_web::web;
use actix_web::HttpResponse;
use surrealdb::{engine::any::Any, Surreal};

#[tracing::instrument]
pub async fn create_account(
    instance: web::Json<CreateAccountDto>,
    db: web::Data<Surreal<Any>>,
) -> HttpResponse {
    tracing::trace!("Reached create_account route handler");
    let resp = match create_account_db(instance, db).await {
        Ok(instance) => HttpResponse::Ok().json(instance),
        Err(_) => HttpResponse::InternalServerError().finish(), // TODO: properly respond to errors, not just send 500
    };
    tracing::trace!("Handler exited");
    resp
}

#[tracing::instrument(skip(db))]
async fn create_account_db(
    account: web::Json<CreateAccountDto>,
    db: web::Data<Surreal<Any>>,
) -> Result<Option<Account>, surrealdb::Error> {
    tracing::info!("Attempting to saving new account to the db");
    let account: CreateAccountDb = account.into_inner().into();

    let account = db
        .create::<Option<Account>>(("account", account.id()))
        .content(account)
        .await
        .map_err(|e| {
            tracing::error!("Failed to persist account to db: {}", e.to_string());
            e
        })?;

    dbg!(&account);
    tracing::info!("Success");
    Ok(account)
}
