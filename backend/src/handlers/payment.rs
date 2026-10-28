use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::error::ApiError;
use crate::middleware::extract_user_id;

pub async fn create_payment(
    req: HttpRequest,
    _pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    let _user_id = extract_user_id(&req)?;
    Ok(HttpResponse::Ok().finish())
}
