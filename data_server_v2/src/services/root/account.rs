use crate::model::account::Account;
use crate::model::account::CreateAccountDb;
use crate::model::account::CreateAccountDto;
use crate::model::CreateTable;
use crate::model::Table;
use crate::services::util::HttpError;
use actix_web::web;
use actix_web::HttpResponse;
use lettre::message::header::ContentType;
use lettre::AsyncSmtpTransport;
use lettre::AsyncTransport;
use lettre::Message;
use lettre::Tokio1Executor;

use surrealdb::{engine::any::Any, Surreal};

#[tracing::instrument(skip(mailer, db))]
pub async fn create_account(
    instance: web::Json<CreateAccountDto>,
    db: web::Data<Surreal<Any>>,
    mailer: web::Data<AsyncSmtpTransport<Tokio1Executor>>,
) -> HttpResponse {
    tracing::trace!("Reached create_account route handler");
    let (resp, account) = match create_account_db(instance, db).await {
        Ok(account) => (HttpResponse::Ok().json(&account), account),
        Err(e) => {
            let e: HttpError = e.into();
            (e.inter_inner(), None)
        }
    };

    if let Some(account) = account {
        match send_confirmation_email(&account, mailer).await {
            Ok(_) => tracing::trace!("Confirmation email send success"),
            Err(e) => {
                tracing::error!("Confirmation email send failed with error: {e}")
                // TODO: remove accounts when email failed to send?
            }
        }
    }

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

#[tracing::instrument(skip(mailer))]
async fn send_confirmation_email(
    account: &Account,
    mailer: web::Data<lettre::AsyncSmtpTransport<Tokio1Executor>>,
) -> Result<(), anyhow::Error> {
    tracing::info!("Attempting to send confirmation email");

    tracing::debug!("Building the email");
    // TODO: make message settings subject to configuration
    let message = Message::builder()
        .from("no-reply <no-reply@rush.io>".parse()?) // TODO: make this configurable
        .to(format!(
            "{} <{}>",
            account
                .name
                .as_ref()
                .expect("Should have received the account name"), // TODO: properly handle this rather than using expect
            account
                .email
                .as_ref()
                .expect("Should have received the email address") // TODO: properly handle this rather than using expect
        )
        .parse()?)
        .subject("Please confirm your email address")
        .header(ContentType::TEXT_PLAIN)
        .body::<String>("Please click the link to confirm your email address.".into())?;
    tracing::debug!("Build success");

    tracing::debug!("Sending email...");
    mailer.send(message).await?; // TODO: consider what we should do if sending the confirmation email fails
    tracing::info!("Confirmation email send success");

    Ok(())
}
