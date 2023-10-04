use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::model::instance::InstanceName;

pub struct VirtualHostProcessor;

impl<S, B> Transform<S, ServiceRequest> for VirtualHostProcessor
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = VirtualHostMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(VirtualHostMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct VirtualHostMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for VirtualHostMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;

    type Error = Error;

    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        tracing::debug_span!("Virtual host extractor middleware");
        tracing::debug!("Checking for instance name in host");
        let srv = self.service.clone();

        async move {
            let instance: Result<InstanceName, _> = req.connection_info().host().try_into();

            if let Ok(instance) = instance {
                tracing::debug!("Instance name found: {instance}");
                req.extensions_mut().insert(instance);
            } else {
                tracing::debug!("No instance name found");
            }

            let res = srv.call(req).await?;

            Ok(res)
        }
        .boxed_local()
    }
}
