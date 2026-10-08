use actix_web::HttpResponse;

pub async fn get_chain() -> Result<HttpResponse, crate::error::ApiError> {
    // TODO: Implement get blockchain chain
    Err(crate::error::ApiError::InternalError)
}

pub async fn get_block(path: actix_web::web::Path<String>) -> Result<HttpResponse, crate::error::ApiError> {
    // TODO: Implement get block
    Err(crate::error::ApiError::InternalError)
}

pub async fn validate_chain() -> Result<HttpResponse, crate::error::ApiError> {
    // TODO: Implement validate chain
    Err(crate::error::ApiError::InternalError)
}
