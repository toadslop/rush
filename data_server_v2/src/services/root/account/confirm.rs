use crate::services::error::{DatabaseError, ErrorResponse};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use serde::Deserialize;
use surrealdb::{engine::any::Any, Surreal};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Parameters {
    #[allow(unused)]
    token: Uuid,
}

#[tracing::instrument]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    db: web::Data<Surreal<Any>>,
) -> Result<HttpResponse, ConfirmAccountError> {
    tracing::trace!("Beginning account confirmation");

    let response = db
        .query(format!("fn::confirm_account('{}')", parameters.token))
        .await
        .map_err(|e| {
            tracing::error!("Failed to to confirm the account: {e}");
            DatabaseError::from(e)
        })?;

    let resp = response
        .check()
        .map(|_| HttpResponse::Ok().finish())
        .map_err(|e| {
            tracing::error!("Failed to to confirm the account: {e}");
            DatabaseError::from(e)
        })?;

    tracing::trace!("Successfully confirmed the account");

    Ok(resp)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfirmAccountError {
    #[error("Failed to persist the account to the database: {0}")]
    DatabaseError(#[from] DatabaseError),
}

impl ResponseError for ConfirmAccountError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmAccountError::DatabaseError(e) => e.status_code(),
        }
    }

    fn error_response(&self) -> HttpResponse {
        let response: ErrorResponse = self.into();
        HttpResponse::build(self.status_code())
            .content_type("application/json")
            .json(response)
    }
}

impl From<&ConfirmAccountError> for ErrorResponse
where
    ConfirmAccountError: ResponseError,
{
    fn from(value: &ConfirmAccountError) -> Self {
        match value {
            ConfirmAccountError::DatabaseError(_) => Self {
                status_code: value.status_code().as_u16(),
                message: value.to_string(),
            },
        }
    }
}
