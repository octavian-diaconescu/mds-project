use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, error::ErrorUnauthorized,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use crate::auth::validate_jwt;

// Struct that represents the middleware
pub struct JwtMiddleware;

// Required by Actix to hook into the middleware system
impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareMiddleware {
            service: Rc::new(service),
        }))
    }
}

// The actual logic of your middleware
pub struct JwtMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    //mut req 
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let token = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            match token {
                Some(token) => match validate_jwt(token) {
                    Ok(claims) => {
                        req.extensions_mut().insert(claims.user_id);
                        service.call(req).await
                    }
                    Err(_) => Err(ErrorUnauthorized("Invalid token")),
                },
                None => Err(ErrorUnauthorized("Missing or malformed Authorization header")),
            }
        })
    }
}
