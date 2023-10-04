use crate::model::instance::Instance;
use actix_web::{web, HttpResponse};
use surrealdb::{engine::any::Any, Error, Surreal};

#[tracing::instrument(
    skip(db),
    fields(
    name = %instance.name,
    )
    )]
pub async fn create_instance(
    instance: web::Json<Instance>,
    db: web::Data<Surreal<Any>>,
) -> HttpResponse {
    tracing::trace!("Reached create_instance route handler");
    let resp = match create_instance_db(instance, db).await {
        Ok(instance) => HttpResponse::Ok().json(instance),
        Err(_) => HttpResponse::InternalServerError().finish(),
    };
    tracing::trace!("Handler exited");
    resp
}

#[tracing::instrument(skip(db))]
async fn create_instance_db(
    instance: web::Json<Instance>,
    db: web::Data<Surreal<Any>>,
) -> Result<Vec<Instance>, Error> {
    tracing::info!("Attempting to saving new instance to the db");
    let instance = db
        .create::<Vec<Instance>>("instance")
        .content(instance)
        .await
        .map_err(|e| {
            tracing::error!("Failed to persist instance to db: {:?}", e);
            e
        })?;
    tracing::info!("Success");
    Ok(instance)
}
