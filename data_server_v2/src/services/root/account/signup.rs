use crate::configuration::app::ApplicationSettings;
use crate::model::account::CreateAccountResp;
use crate::services::error::ErrorResponse;
use crate::{
    model::account::{CreateAccountDb, CreateAccountDto},
    services::error::DatabaseError,
};
use actix_web::http::uri::InvalidUri;
use actix_web::http::StatusCode;
use actix_web::{http::Uri, web, HttpResponse, ResponseError};
use lettre::address::AddressError;
use lettre::{
    message::header::ContentType, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use surrealdb::opt::auth::Root;
use surrealdb::{engine::any::Any, Surreal};
use uuid::Uuid;

#[tracing::instrument(skip(mailer, db))]
pub async fn create_account(
    instance: web::Json<CreateAccountDto>,
    db: web::Data<Surreal<Any>>,
    mailer: web::Data<AsyncSmtpTransport<Tokio1Executor>>,
    settings: web::Data<ApplicationSettings>,
) -> Result<HttpResponse, CreateAccountError> {
    tracing::trace!("Reached create_account route handler");
    db.signin(Root {
        // TODO: handle root db signin for specific endpoints
        username: "root",
        password: "root",
    })
    .await
    .unwrap();
    let (resp, account) = match create_account_db(instance, db).await {
        Ok(account) => (
            HttpResponse::Ok().json(
                &account
                    .as_ref()
                    .ok_or(CreateAccountError::MissingAccountError)?
                    .account,
            ),
            account,
        ),
        Err(e) => {
            tracing::error!("Confirmation email send failed with error: {e}");
            (HttpResponse::BadRequest().finish(), None)
        }
    };

    if let Some(account) = account {
        match send_confirmation_email(&account, mailer, settings).await {
            Ok(_) => tracing::trace!("Confirmation email send success"),
            Err(e) => {
                tracing::error!("Confirmation email send failed with error: {e}");
                return Err(e);
                // TODO: remove accounts when email failed to send?
            }
        }
    }
    dbg!(&resp);
    tracing::trace!("Handler exited");
    Ok(resp)
}

#[tracing::instrument(skip(db))]
async fn create_account_db(
    account: web::Json<CreateAccountDto>,
    db: web::Data<Surreal<Any>>,
) -> Result<Option<CreateAccountResp>, CreateAccountError> {
    tracing::info!("Attempting to saving new account to the db");
    let account: CreateAccountDb = account.into_inner().into();

    let result = db // TODO: refactor to use the transaction wrapper class
        .query(r#"
            BEGIN TRANSACTION;
            LET $saved = CREATE ONLY account CONTENT {
                email: $email,
                password: crypto::argon2::generate($password),
                name: $name,
                id: $id
            };
            LET $conf_token = (SELECT ->has->confirmation_token.token FROM $saved)[0]["->has"]["->confirmation_token"].token[0];
            RETURN {
                account: $saved,
                token: $conf_token
            };
            COMMIT TRANSACTION;
        "#,
        )
        .bind(account)
        .await.map_err(DatabaseError::from)?;

    let mut result = result
        .check()
        .map_err(|e| {
            tracing::error!("Failed to persist account to db: {e}");
            e
        })
        .map_err(DatabaseError::from)?;

    let account_resp: Option<CreateAccountResp> = result.take(0).map_err(DatabaseError::from)?;

    tracing::info!("Success");
    Ok(account_resp)
}

fn build_confirmation_link(
    settings: &ApplicationSettings,
    token: &Uuid,
) -> Result<Uri, InvalidUri> {
    let ApplicationSettings {
        port, host, scheme, ..
    } = settings;
    Uri::try_from(format!(
        "{}://{}:{}/account/confirm?token={}",
        scheme, host, port, token
    ))
}

#[tracing::instrument(skip(mailer, settings))]
async fn send_confirmation_email(
    account: &CreateAccountResp,
    mailer: web::Data<lettre::AsyncSmtpTransport<Tokio1Executor>>,
    settings: web::Data<ApplicationSettings>,
) -> Result<(), CreateAccountError> {
    tracing::info!("Attempting to send confirmation email");

    let confirmation_link = build_confirmation_link(&settings, &account.token)?;

    // TODO: make message settings subject to configuration
    let message = Message::builder() // TODO: wrap this in a custom message builder
        .from("no-reply <no-reply@rush.io>".parse()?) // TODO: add this to configuration and parse it at startup, not here
        .to(format!(
            "{} <{}>",
            account
                .account
                .name
                .as_ref()
                .ok_or(CreateAccountError::MissingAccountName)?,
            account
                .account
                .email
                .as_ref()
                .ok_or(CreateAccountError::MissingAccountEmail)?
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

#[derive(Debug, thiserror::Error)]
pub enum CreateAccountError {
    #[error("Failed to persist the account to the database: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Failed to construct valid email confirmation uri: {0}")]
    InvalidUri(#[from] InvalidUri),
    #[error("Expected to have received the account name from the server but found None")]
    MissingAccountName,
    #[error("Expected to have received the account email address from the server but found None")]
    MissingAccountEmail,
    #[error("The system address is invalid and cannot be parsed: {0}")]
    InvalidSystemEmail(#[from] AddressError),
    #[error("Failed to construct email message: {0}")]
    MailConstructionError(#[from] lettre::error::Error),
    #[error("Failed to send the email message: {0}")]
    MailSendError(#[from] lettre::transport::smtp::Error),
    #[error("No account was returned from the database")]
    MissingAccountError,
}

impl ResponseError for CreateAccountError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateAccountError::DatabaseError(e) => e.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let response: ErrorResponse = self.into();
        HttpResponse::build(self.status_code())
            .content_type("application/json")
            .json(response)
    }
}

impl From<&CreateAccountError> for ErrorResponse
where
    CreateAccountError: ResponseError,
{
    fn from(value: &CreateAccountError) -> Self {
        match value {
            CreateAccountError::DatabaseError(_) => Self {
                status_code: value.status_code().as_u16(),
                message: value.to_string(),
            },
            _ => Default::default(),
        }
    }
}
