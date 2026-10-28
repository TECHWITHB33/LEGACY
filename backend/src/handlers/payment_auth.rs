use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::error::ApiError;
use crate::payment_gateway::*;
use crate::stellar_payment::*;
use crate::models::{Credential, CredentialStatus};
use crate::biometric_auth::AuthenticationChallenge;
use chrono::Utc;

pub async fn authorize_payment(
    _body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().finish())
}
