use crate::model::account::Account;
use crate::model::account::CreateAccountDb;
use crate::model::account::CreateAccountDto;
use crate::model::CreateTable;
use crate::model::Table;
use crate::services::util::HttpError;
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
        Err(e) => {
            let e: HttpError = e.into();
            e.inter_inner()
        }
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
        .create::<Option<Account>>((Account::name(), account.id()))
        .content(account)
        .await
        .map_err(|e| {
            tracing::error!("Failed to persist account to db: {e}");
            e
        })?;

    tracing::info!("Success");
    Ok(account)
}
