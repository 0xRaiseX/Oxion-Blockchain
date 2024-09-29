use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use actix_service::{Service, Transform};
use futures::future::{ok, Ready};
use std::task::{Context, Poll};
use std::pin::Pin;

pub struct ContentTypeJson;

impl<S, B> Transform<S, ServiceRequest> for ContentTypeJson
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ContentTypeJsonMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ContentTypeJsonMiddleware { service })
    }
}

pub struct ContentTypeJsonMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ContentTypeJsonMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut req = req;
        if req.headers().get("Content-Type").is_none() {
            req.headers_mut().insert(
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::HeaderValue::from_static("application/json"),
            );
        }
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
