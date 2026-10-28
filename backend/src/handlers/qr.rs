use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::Arc;
use crate::error::ApiError;
use crate::models::Credential;
use crate::qr_verification::{SecureQrData, QrVerificationEngine};
use crate::services::QrService;
use crate::middleware::extract_user_id;
use sqlx::SqlitePool;

pub async fn generate_qr(
    req: HttpRequest,
    path: web::Path<String>,
    _pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    let _user_id = extract_user_id(&req)?;
    let _credential_id = path.into_inner();
    Ok(HttpResponse::Ok().finish())
}
