use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::models::*;
use crate::error::ApiError;
use crate::config::Config;

pub async fn register(
    _body: web::Json<RegisterRequest>,
    _pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn login(
    _body: web::Json<LoginRequest>,
    _pool: web::Data<Arc<SqlitePool>>,
    _config: web::Data<Arc<Config>>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().finish())
}
