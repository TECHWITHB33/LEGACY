use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::error::ApiError;
use crate::models::*;
use crate::middleware::extract_user_id;
use crate::services::CredentialIssuerService;
use crate::validators;
use crate::crypto::hash::*;
use crate::crypto::signature::*;
use crate::credential_expiration::ExpirationManager;
use chrono::{Duration, Utc};

pub async fn create_credential(
    req: HttpRequest,
    body: web::Json<CreateCredentialRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    let _user_id = extract_user_id(&req)?;
    Ok(HttpResponse::Ok().finish())
}
