use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::Arc;
use crate::error::ApiError;
use crate::biometric_auth::*;
use crate::middleware::extract_user_id;
use chrono::Utc;
use uuid::Uuid;
use sqlx::SqlitePool;

pub async fn initiate_biometric_auth(
    req: HttpRequest,
    _pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    let _user_id = extract_user_id(&req)?;
    Ok(HttpResponse::Ok().finish())
}
