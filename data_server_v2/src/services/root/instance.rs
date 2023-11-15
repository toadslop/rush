use crate::{
    model::{
        instance::{CreateInstanceDb, CreateInstanceDto, Instance},
        CreateTable, Table,
    },
    services::error::{DatabaseError, ErrorResponse},
};
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use surrealdb::{engine::any::Any, Surreal};

#[tracing::instrument(skip(db))]
pub async fn create_instance(
    instance: web::Json<CreateInstanceDto>,
    db: web::Data<Surreal<Any>>,
) -> Result<HttpResponse, CreateInstanceError> {
    tracing::info!("Reached create_instance route handler");

    let resp = create_instance_db(instance, db).await?;

    tracing::info!("Handler exited");

    Ok(HttpResponse::Ok().json(resp))
}

#[tracing::instrument(skip(db))]
async fn create_instance_db(
    instance: web::Json<CreateInstanceDto>,
    db: web::Data<Surreal<Any>>,
) -> Result<Instance, CreateInstanceError> {
    tracing::info!("Attempting to saving new instance to the db");
    let instance: CreateInstanceDb = instance.into_inner().into();

    let instance = db
        .create::<Option<Instance>>((Instance::name(), instance.id()))
        .content(instance)
        .await
        .map_err(DatabaseError::from)?
        .ok_or(CreateInstanceError::AuthenticationFailure)?;

    tracing::info!("Success: {:?}", instance);

    Ok(instance)
}

#[derive(Debug, thiserror::Error)]
pub enum CreateInstanceError {
    #[error("Failed to persist the instance to the database: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("User must be logged-in to create an instance")]
    AuthenticationFailure,
    #[error("Failed to persist the submitted instance")]
    _NotPersisted,
}

impl ResponseError for CreateInstanceError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateInstanceError::DatabaseError(e) => e.status_code(),
            CreateInstanceError::AuthenticationFailure => StatusCode::UNAUTHORIZED,
            CreateInstanceError::_NotPersisted => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let response: ErrorResponse = self.into();
        HttpResponse::build(self.status_code())
            .content_type("application/json")
            .json(response)
    }
}

impl From<&CreateInstanceError> for ErrorResponse
where
    CreateInstanceError: ResponseError,
{
    fn from(value: &CreateInstanceError) -> Self {
        Self {
            status_code: value.status_code().as_u16(),
            message: value.to_string(),
        }
    }
}
