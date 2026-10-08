use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpRequest, HttpResponse,
};
use futures::future::LocalBoxFuture;
use std::rc::Rc;
use crate::error::ApiError;
use crate::jwt::{self, JwtClaims};
use crate::config::Config;

/// Extract user ID from JWT token in request
pub fn extract_user_id(req: &HttpRequest) -> Result<String, ApiError> {
    req.extensions()
        .get::<JwtClaims>()
        .ok_or_else(|| ApiError::AuthenticationError(
            "No JWT claims found in request".to_string()
        ))
        .map(|claims| claims.sub.clone())
}

pub struct CorsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for CorsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CorsMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(CorsMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct CorsMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CorsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        Box::pin(async move {
            let res = srv.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

/// JWT authentication middleware factory
pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract Authorization header
        let auth_header = req.headers().get("Authorization");

        let result = match auth_header {
            Some(header) => {
                match header.to_str() {
                    Ok(header_str) => {
                        // Extract token from header
                        match jwt::extract_token_from_header(header_str) {
                            Ok(token) => {
                                // Get config from app data
                                let config = req.app_data::<actix_web::web::Data<std::sync::Arc<Config>>>();
                                
                                match config {
                                    Some(config) => {
                                        // Validate token
                                        match jwt::validate_token(&token, &config) {
                                            Ok(claims) => {
                                                // Store claims in request extensions
                                                req.extensions_mut().insert(claims);
                                                Ok(())
                                            }
                                            Err(_) => Err(ApiError::AuthenticationError("Invalid token".to_string())),
                                        }
                                    }
                                    None => Err(ApiError::InternalError),
                                }
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Err(_) => Err(ApiError::AuthenticationError("Invalid header encoding".to_string())),
                }
            }
            None => Err(ApiError::AuthenticationError("Missing Authorization header".to_string())),
        };

        let srv = self.service.clone();

        Box::pin(async move {
            match result {
                Ok(()) => {
                    let res = srv.call(req).await?;
                    Ok(res.map_into_left_body())
                }
                Err(api_error) => {
                    let error_response = api_error.error_response();
                    Ok(req.error_response(error_response).map_into_left_body())
                }
            }
        })
    }
}
