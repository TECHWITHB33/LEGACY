pub mod authentication;
pub mod students;
pub mod credentials;
pub mod verification;
pub mod cryptography;
pub mod qr;
pub mod payments;
pub mod audit_service;
pub mod credential_issuer;
pub mod escrow_service;

pub use authentication::AuthenticationService;
pub use students::StudentService;
pub use credentials::CredentialService;
pub use verification::VerificationService;
pub use cryptography::CryptographyService;
pub use qr::QrService;
pub use payments::PaymentService;
pub use audit_service::AuditService;
pub use credential_issuer::CredentialIssuerService;
pub use escrow_service::EscrowService;

use sqlx::SqlitePool;
use std::sync::Arc;
use crate::config::Config;

/// Application services container
pub struct Services {
    pub authentication: Arc<AuthenticationService>,
    pub students: Arc<StudentService>,
    pub credentials: Arc<CredentialService>,
    pub verification: Arc<VerificationService>,
    pub cryptography: Arc<CryptographyService>,
    pub qr: Arc<QrService>,
    pub payments: Arc<PaymentService>,
    pub audit: Arc<AuditService>,
    pub escrow: Arc<EscrowService>,
}

impl Services {
    pub fn new(pool: Arc<SqlitePool>, config: Arc<Config>) -> Self {
        Services {
            authentication: Arc::new(AuthenticationService::new(pool.clone(), config.clone())),
            students: Arc::new(StudentService::new(pool.clone())),
            credentials: Arc::new(CredentialService::new(pool.clone())),
            verification: Arc::new(VerificationService::new(pool.clone())),
            cryptography: Arc::new(CryptographyService::new()),
            qr: Arc::new(QrService::new()),
            payments: Arc::new(PaymentService::new(pool.clone())),
            audit: Arc::new(AuditService::new(pool.clone())),
            escrow: Arc::new(EscrowService::new(pool.clone(), config.clone())),
        }
    }
}
