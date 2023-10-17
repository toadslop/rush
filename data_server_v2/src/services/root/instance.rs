use crate::{
    model::{
        instance::{CreateInstanceDb, CreateInstanceDto, Instance},
        CreateTable, Table,
    },
    services::util::HttpError,
};
use actix_web::{web, HttpResponse};
use surrealdb::{engine::any::Any, Surreal};

#[tracing::instrument(skip(db))]
pub async fn create_instance(
    instance: web::Json<CreateInstanceDto>,
    db: web::Data<Surreal<Any>>,
) -> HttpResponse {
    tracing::trace!("Reached create_instance route handler");

    let resp = match create_instance_db(instance, db).await {
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
async fn create_instance_db(
    instance: web::Json<CreateInstanceDto>,
    db: web::Data<Surreal<Any>>,
) -> Result<Option<Instance>, surrealdb::Error> {
    tracing::info!("Attempting to saving new instance to the db");
    let instance: CreateInstanceDb = instance.into_inner().into();
    dbg!(&instance);
    dbg!(serde_json::to_string(&instance).unwrap());
    let instance = db
        .create::<Option<Instance>>((Instance::name(), instance.id()))
        .content(instance)
        .await
        .map_err(|e| {
            tracing::error!("Failed to persist instance to db: {e}");
            e
        })?;
    tracing::info!("Success");
    Ok(instance)
}
