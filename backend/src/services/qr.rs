use crate::error::ApiError;

pub struct QrService;

impl QrService {
    pub fn new() -> Self {
        QrService
    }

    /// Generate QR code from data
    pub fn generate(&self, data: &str) -> Result<Vec<u8>, ApiError> {
        let qr_code = qrcode::QrCode::new(data)
            .map_err(|e| ApiError::QrError(format!("QR generation failed: {}", e)))?;

        let image = qr_code.render::<qrcode::render::svg::Color>();
        let svg_string = image.build().as_bytes().to_vec();

        Ok(svg_string)
    }

    /// Hash QR code data
    pub fn hash_qr(&self, data: &str) -> String {
        crate::crypto::sha256_hex(data.as_bytes())
    }

    /// Verify QR hash
    pub fn verify_hash(&self, data: &str, expected_hash: &str) -> bool {
        let computed_hash = self.hash_qr(data);
        computed_hash == expected_hash
    }
}
