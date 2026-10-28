use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::error::ApiError;
use crate::stellar_payment::*;

pub async fn create_payment(
    _req: HttpRequest,
    _body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().finish())
}
