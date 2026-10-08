use actix_web::HttpResponse;

pub async fn list_payments() -> Result<HttpResponse, crate::error::ApiError> {
    // TODO: Implement list payments
    Err(crate::error::ApiError::InternalError)
}

pub async fn create_payment() -> Result<HttpResponse, crate::error::ApiError> {
    // TODO: Implement create payment
    Err(crate::error::ApiError::InternalError)
}

pub async fn get_payment(path: actix_web::web::Path<String>) -> Result<HttpResponse, crate::error::ApiError> {
    // TODO: Implement get payment
    Err(crate::error::ApiError::InternalError)
}

pub async fn verify_payment(path: actix_web::web::Path<String>) -> Result<HttpResponse, crate::error::ApiError> {
    // TODO: Implement verify payment
    Err(crate::error::ApiError::InternalError)
}

pub async fn generate_merchant_qr() -> Result<HttpResponse, crate::error::ApiError> {
    // TODO: Implement merchant QR generation
    Err(crate::error::ApiError::InternalError)
}
