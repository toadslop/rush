use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Path,
    Error,
};
use futures_util::future::LocalBoxFuture;

pub struct PipeManager;

impl<S, B> Transform<S, ServiceRequest> for PipeManager
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PipeMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PipeMiddleware { service }))
    }
}

pub struct PipeMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for PipeMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());

        let matchpattern = req.match_pattern().unwrap();

        let matchinfo = req.match_info();
        let thing = req.match_info(); //.load::<(String, String)>();
        println!("{:?}", matchpattern);
        println!("{:?}", thing);
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            println!("Hi from response");
            Ok(res)
        })
    }
}
