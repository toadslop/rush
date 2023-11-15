use actix_web::http::StatusCode;
use actix_web::ResponseError;
use serde::Serialize;
use std::fmt::Display;

#[derive(thiserror::Error)]
pub struct DatabaseError(#[from] surrealdb::Error);

impl ResponseError for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match &self.0 {
            surrealdb::Error::Db(e) => match e {
                surrealdb::error::Db::FieldCheck { .. } => StatusCode::BAD_REQUEST,
                surrealdb::error::Db::FieldValue { .. } => StatusCode::BAD_REQUEST,
                surrealdb::error::Db::TxFailure => StatusCode::BAD_REQUEST,
                surrealdb::error::Db::QueryNotExecuted => StatusCode::BAD_REQUEST,
                surrealdb::error::Db::SingleOnlyOutput => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            surrealdb::Error::Api(e) => match e {
                surrealdb::error::Api::Query(..) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            surrealdb::Error::Db(e) => write!(f, "{e}"),
            surrealdb::Error::Api(e) => write!(f, "{e}"),
        }
    }
}

impl std::fmt::Debug for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(&self.0, f)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status_code: u16,
    pub message: String,
}

impl Default for ErrorResponse {
    fn default() -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "An internal server error occurred".into(),
        }
    }
}

// Note to self: relying on surrealdb's assertions for validations will lead to the following problems:
// 1. difficulty providing user friendly error messages
// 2. only returns one error when possibly multiple fields have errors
// 3. cannot execute server side
// 4. cannot separate multiple validations, each with a distinct error message
// Possible solutions
// 1. Make PR directly to surrealdb to modify how errors are handled
// 2. implement my own validation system from scratch <--
//    maybe create a 'validations' table in surrealdb
//    generate assert clauses from the table
//    read the table to validate client side and server side
