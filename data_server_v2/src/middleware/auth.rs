use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use surrealdb::{engine::any::Any, Surreal};

pub struct AuthProcessor;

impl<S, B> Transform<S, ServiceRequest> for AuthProcessor
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;

    type Error = Error;

    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        tracing::debug_span!("Db auth middlware");
        tracing::debug!("Checking auth");

        let srv = self.service.clone();

        async move {
            let creds = match req.extract::<BearerAuth>().await {
                Ok(creds) => creds,
                Err(_) => {
                    tracing::debug!("No creds found. Authenticating as guest");
                    let db = req
                        .app_data::<web::Data<Surreal<Any>>>()
                        .expect("Should have the db")
                        .clone(); // TODO: handle error properly

                    db.invalidate()
                        .await
                        .expect("Failed to invalidate connection");
                    let res = srv.call(req).await?;
                    return Ok(res);
                }
            };

            let db = req
                .app_data::<web::Data<Surreal<Any>>>()
                .expect("Should have the db")
                .clone(); // TODO: handle error properly

            dbg!(creds.token());
            match db.authenticate(creds.token()).await {
                Ok(_) => {}
                Err(e) => println!("HERE {e}"),
            }; // TODO: handle error properly

            let res = srv.call(req).await?;

            db.invalidate()
                .await
                .expect("Failed to invalidate connection");

            Ok(res)
        }
        .boxed_local()
    }
}
