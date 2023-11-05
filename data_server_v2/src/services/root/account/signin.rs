use crate::services::error::DatabaseError;
use crate::{model::account::AccountSignin, services::error::ErrorResponse};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use surrealdb::opt::auth::Scope;
use surrealdb::{engine::any::Any, Surreal};

#[tracing::instrument(skip(db))]
pub async fn sign_in(
    signin_info: web::Json<AccountSignin>,
    db: web::Data<Surreal<Any>>,
) -> Result<HttpResponse, SignInError> {
    tracing::info!("Signin Requested");

    let jwt = db
        .signin(Scope::<AccountSignin> {
            namespace: "root", // TODO: get from config
            database: "root",  // TODO: get from config
            scope: "account",  // TODO: get from config
            params: signin_info.into_inner(),
        })
        .await
        .map_err(DatabaseError::from)?;

    tracing::info!("Signin Success");
    Ok(HttpResponse::build(StatusCode::OK).json(jwt))
}

#[derive(Debug, thiserror::Error)]
pub enum SignInError {
    #[error("Signin failed: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("No account was returned from the database")]
    _UnknownError,
}

impl ResponseError for SignInError {
    fn status_code(&self) -> StatusCode {
        match self {
            SignInError::DatabaseError(e) => e.status_code(),
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

impl From<&SignInError> for ErrorResponse
where
    SignInError: ResponseError,
{
    fn from(value: &SignInError) -> Self {
        match value {
            SignInError::DatabaseError(_) => Self {
                status_code: value.status_code().as_u16(),
                message: value.to_string(),
            },
            _ => Default::default(),
        }
    }
}
