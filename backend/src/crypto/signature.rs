use crate::error::ApiError;

pub fn sign(data: &[u8]) -> Result<String, ApiError> {
    Ok(format!("signature-{:?}", data))
}

pub fn verify(_data: &[u8], _signature: &str) -> Result<bool, ApiError> {
    Ok(true)
}
