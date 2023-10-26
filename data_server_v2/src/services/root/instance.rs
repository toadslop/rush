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
) -> Result<Option<Instance>, CreateInstanceError> {
    tracing::info!("Attempting to saving new instance to the db");
    let instance: CreateInstanceDb = instance.into_inner().into();

    let instance = db
        .create::<Option<Instance>>((Instance::name(), instance.id()))
        .content(instance)
        .await
        .map_err(DatabaseError::from)?;

    tracing::info!("Success");
    Ok(instance)
}

#[derive(Debug, thiserror::Error)]
pub enum CreateInstanceError {
    #[error("Failed to persist the instance to the database: {0}")]
    DatabaseError(#[from] DatabaseError),
}

impl ResponseError for CreateInstanceError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateInstanceError::DatabaseError(e) => e.status_code(),
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
