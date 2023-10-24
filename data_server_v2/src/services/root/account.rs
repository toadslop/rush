use crate::model::account::Account;
use crate::model::account::CreateAccountDb;
use crate::model::account::CreateAccountDto;
use crate::services::util::HttpError;
use crate::AppAddress;
use actix_web::http::Uri;
use actix_web::web;
use actix_web::HttpResponse;
use lettre::message::header::ContentType;
use lettre::AsyncSmtpTransport;
use lettre::AsyncTransport;
use lettre::Message;
use lettre::Tokio1Executor;

use serde::Deserialize;
use surrealdb::{engine::any::Any, Surreal};
use uuid::Uuid;

#[tracing::instrument(skip(mailer, db, app_address))]
pub async fn create_account(
    instance: web::Json<CreateAccountDto>,
    db: web::Data<Surreal<Any>>,
    mailer: web::Data<AsyncSmtpTransport<Tokio1Executor>>,
    app_address: web::Data<AppAddress>,
) -> HttpResponse {
    tracing::trace!("Reached create_account route handler");
    let (resp, account) = match create_account_db(instance, db).await {
        Ok(account) => (
            HttpResponse::Ok().json(&account.as_ref().unwrap().account),
            account,
        ),
        Err(e) => {
            let e: HttpError = e.into();
            (e.inter_inner(), None)
        }
    };

    if let Some(account) = account {
        match send_confirmation_email(&account, mailer, app_address).await {
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

#[derive(Debug, Deserialize)]
struct Res {
    account: Account,
    token: Uuid,
}

#[tracing::instrument(skip(db))]
async fn create_account_db(
    account: web::Json<CreateAccountDto>,
    db: web::Data<Surreal<Any>>,
) -> Result<Option<Res>, surrealdb::Error> {
    tracing::info!("Attempting to saving new account to the db");
    let account: CreateAccountDb = account.into_inner().into();

    let result = db
        .query(r#"
            BEGIN TRANSACTION;
            LET $saved = CREATE ONLY account CONTENT $account;
            LET $conf_token = (SELECT ->has->confirmation_token.token FROM $saved)[0]["->has"]["->confirmation_token"].token[0];
            RETURN {
                account: $saved,
                token: $conf_token
            };
            COMMIT TRANSACTION;
        "#,
        )
        .bind(("account", account))
        .await
        .unwrap();

    let mut result = result.check().map_err(|e| {
        tracing::error!("Failed to persist account to db: {e}");
        e
    })?;
    let thing: Option<Res> = result.take(0).unwrap();

    // let account = db
    //     .create::<Option<Account>>((Account::name(), account.id()))
    //     .content(account)
    //     .await
    // .map_err(|e| {
    //     tracing::error!("Failed to persist account to db: {e}");
    //     e
    // })?;

    tracing::info!("Success");
    Ok(thing)
}

#[tracing::instrument(skip(mailer, app_address))]
async fn send_confirmation_email(
    account: &Res,
    mailer: web::Data<lettre::AsyncSmtpTransport<Tokio1Executor>>,
    app_address: web::Data<AppAddress>,
) -> Result<(), anyhow::Error> {
    tracing::info!("Attempting to send confirmation email");

    tracing::debug!("Building the email");
    let endpoint = format!(
        "http://{}/account/confirm?token={}",
        app_address.0,
        account.token // TODO: actually get the generated UUID
    ); // TODO: handle https
    let confirmation_link = Uri::try_from(endpoint).unwrap();

    // TODO: make message settings subject to configuration
    let message = Message::builder()
        .from("no-reply <no-reply@rush.io>".parse()?) // TODO: make this configurable
        .to(format!(
            "{} <{}>",
            account
                .account
                .name
                .as_ref()
                .expect("Should have received the account name"), // TODO: properly handle this rather than using expect
            account
                .account
                .email
                .as_ref()
                .expect("Should have received the email address") // TODO: properly handle this rather than using expect
        )
        .parse()?)
        .subject("Please confirm your email address")
        .header(ContentType::TEXT_PLAIN)
        .body::<String>(format!(
            r#"Please click <a href="{}">here</a> to confirm your email address."#,
            confirmation_link
        ))?;
    tracing::debug!("Build success");

    tracing::debug!("Sending email...");
    mailer.send(message).await?; // TODO: consider what we should do if sending the confirmation email fails
    tracing::info!("Confirmation email send success");

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Parameters {
    #[allow(unused)]
    token: Uuid,
}

#[tracing::instrument(name = "Confirm an account")]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    db: web::Data<Surreal<Any>>,
) -> HttpResponse {
    tracing::trace!("Beginning account confirmation");
    dbg!(&parameters.token);
    let response = match db
        .query(format!("fn::confirm_account('{}')", parameters.token))
        .await
    {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Failed to to confirm the account: {e}");
            let e: HttpError = e.into();
            return e.inter_inner();
        }
    };
    dbg!(&response);
    match response.check() {
        Ok(_) => {
            tracing::trace!("Successfully confirmed the account");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to to confirm the account: {e}");
            let e: HttpError = e.into();
            e.inter_inner()
        }
    }
}
