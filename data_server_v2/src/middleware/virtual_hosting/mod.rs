use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};

use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use std::{
    fmt::Display,
    future::{ready, Ready},
    rc::Rc,
};

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

#[derive(Debug)]
struct InstanceName(String);

impl Display for InstanceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TODO: eventually, we'll need to be able to handle any domain sent to this url
// We will need a mapping of urls to instance names so that we can identify
// which instance a domain is associated with

impl TryFrom<&str> for InstanceName {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut period_count: u8 = 0;
        let mut instance_name = None;
        for (idx, char) in value.chars().enumerate() {
            if char == '.' {
                if period_count == 0 {
                    instance_name = Some(InstanceName(value[0..idx].into()));
                }
                period_count += 1
            }
        }

        if period_count == 2 {
            return instance_name.ok_or("Failed to parse the host");
        }

        Err("Failed to parse the host")
    }
}
