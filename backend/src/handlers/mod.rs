pub mod auth;
pub mod credentials;
pub mod blockchain;
pub mod qr;
pub mod payment;
pub mod biometric;
pub mod payment_auth;
pub mod admin;
pub mod stellar;
pub mod escrow;

use actix_web::HttpResponse;
use crate::models::HealthCheckResponse;
use chrono::Utc;

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthCheckResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
