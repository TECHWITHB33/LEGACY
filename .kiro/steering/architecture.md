---
inclusion: auto
name: Architecture Guide
description: System architecture, design patterns, and best practices for this project
---

# ID Card System - Architecture & Development Guide

## Project Overview

This is a **production-grade Student Identity Card Security System** featuring:
- **Rust backend** with Actix-web framework
- **Streamlit + React frontends**
- **Blockchain verification** with SHA256
- **RSA 4096-bit cryptography**
- **Stellar testnet payments**
- **WebAuthn biometric authentication**
- **Comprehensive audit logging**

## File Organization

```
.
├── backend/                    # Rust API server
│   ├── src/
│   │   ├── main.rs            # Entry point, route configuration
│   │   ├── config.rs          # Configuration management
│   │   ├── error.rs           # Error types and handling
│   │   ├── models.rs          # Request/response data models
│   │   ├── utils.rs           # Utility functions
│   │   ├── crypto/            # Cryptographic modules
│   │   │   ├── rsa.rs         # RSA 4096 key management
│   │   │   ├── hash.rs        # SHA256 hashing
│   │   │   ├── signature.rs   # EdDSA signing
│   │   │   └── key_derivation.rs # PBKDF2
│   │   ├── database/          # Data access layer
│   │   │   ├── user.rs        # User CRUD operations
│   │   │   ├── credential.rs  # Credential management
│   │   │   ├── blockchain.rs  # Blockchain storage
│   │   │   ├── payment.rs     # Payment records
│   │   │   └── audit_log.rs   # Audit trail
│   │   ├── blockchain/        # Blockchain implementation
│   │   ├── handlers/          # API route handlers
│   │   │   ├── auth.rs        # Authentication endpoints
│   │   │   ├── credentials.rs # Credential endpoints
│   │   │   ├── blockchain.rs  # Blockchain endpoints
│   │   │   ├── payment.rs     # Payment endpoints
│   │   │   ├── qr.rs          # QR code endpoints
│   │   │   ├── biometric.rs   # Biometric endpoints
│   │   │   └── admin.rs       # Admin endpoints
│   │   └── middleware.rs      # Custom middleware
│   ├── migrations/            # Database migrations
│   └── Cargo.toml             # Rust dependencies
│
├── frontend_streamlit/        # Streamlit MVP
│   ├── app.py                 # Main application
│   ├── requirements.txt       # Python dependencies
│   └── .env.example           # Environment template
│
├── frontend_react/            # React production UI
│   ├── src/
│   ├── package.json           # Node dependencies
│   └── vite.config.ts         # Build configuration
│
├── docker/                    # Docker configurations
│   ├── backend.Dockerfile     # Backend image
│   ├── streamlit.Dockerfile   # Streamlit image
│   ├── react.Dockerfile       # React image
│   └── nginx.conf             # Nginx config
│
├── .github/workflows/         # CI/CD pipelines
│   ├── ci.yml                 # Testing and building
│   └── deploy.yml             # Deployment automation
│
└── docs/                      # Documentation
    ├── ARCHITECTURE.md        # System design
    ├── SECURITY.md            # Security guide
    ├── SETUP.md               # Development setup
    ├── API.md                 # API reference
    └── DEPLOYMENT.md          # Production deployment
```

## Technology Stack

### Backend (Rust)
- **Framework**: Actix-web 4.4
- **Async Runtime**: Tokio
- **Database**: SQLite + SQLx
- **Cryptography**: sha2, ed25519-dalek, rsa
- **Serialization**: serde, serde_json
- **Logging**: tracing, tracing-subscriber

### Frontend (Streamlit)
- **Framework**: Streamlit 1.28+
- **HTTP**: requests, httpx
- **Crypto**: pycryptodome
- **QR**: qrcode
- **Vision**: opencv-python, pyzbar

### Frontend (React)
- **Build**: Vite 5.0
- **UI Framework**: React 18.2
- **Routing**: React Router 6.18
- **HTTP Client**: Axios 1.6
- **State**: Zustand 4.4
- **Auth**: @webauthn/browser

### Deployment
- **Containers**: Docker & Docker Compose
- **Orchestration**: Kubernetes (optional)
- **CI/CD**: GitHub Actions
- **Cloud**: Any Docker-compatible host

## Key Design Patterns

### 1. Layered Architecture
```
Presentation Layer
    ↓
API Layer (REST)
    ↓
Business Logic Layer
    ↓
Data Access Layer
    ↓
Database Layer
```

### 2. Error Handling
All errors propagate through `ApiError` enum for consistent HTTP responses:

```rust
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    // ... more variants
}
```

### 3. Type Safety
- Strong typing throughout Rust backend
- TypeScript for React frontend
- Serde for serialization guarantees

### 4. Async/Await
All I/O operations are non-blocking:
```rust
pub async fn register(/* ... */) -> Result<HttpResponse, ApiError> {
    // Non-blocking operations only
}
```

## Development Workflow

### Backend Development
1. Make changes in `backend/src/`
2. Run `cargo test` to verify
3. Run `cargo clippy` to check code quality
4. Format with `cargo fmt`
5. Build with `cargo build --release`

### Frontend Development (Streamlit)
1. Edit `frontend_streamlit/app.py`
2. Streamlit auto-reloads on save
3. Test in browser at `http://localhost:8501`

### Frontend Development (React)
1. Edit components in `frontend_react/src/`
2. Vite auto-reloads on save
3. View changes at `http://localhost:5173`

## Important Concepts

### Cryptographic Flow
```
User Registration:
1. Generate RSA 4096 keypair
2. Hash password with PBKDF2-SHA256 (100k iterations)
3. Encrypt private key with derived key
4. Store encrypted private key in DB
5. Create EdDSA signature keypair
6. Generate QR code for public key

Authentication:
1. Verify email exists
2. Verify password hash
3. Issue JWT token (1 hour expiry)
4. Return refresh token (30 day expiry)
```

### Blockchain Security
```
Block Creation:
1. Hash all credential data with SHA256
2. Create block with reference to previous block
3. Mine block (proof-of-work)
4. Validate block integrity
5. Add to immutable chain
```

### Payment Processing
```
Payment Flow:
1. User initiates payment with amount + recipient
2. Create payment record (pending status)
3. Generate unsigned Stellar transaction
4. Display for user review
5. User confirms in UI
6. Sign transaction with private key
7. Submit to Stellar network
8. Poll for confirmation
9. Update payment status
10. Audit log entry
```

## Code Quality Standards

### Formatting
- Rust: `cargo fmt`
- Python: `black --line-length=100`
- TypeScript: `prettier`

### Linting
- Rust: `cargo clippy` with no warnings
- Python: `flake8`, `pylint`
- TypeScript: `eslint`

### Testing
- All public functions must have unit tests
- Integration tests for API endpoints
- E2E tests for critical flows
- Aim for 80%+ code coverage

### Documentation
- Public functions must have rustdoc comments
- Complex logic must be explained
- README in each major directory

## Security Considerations

### Never Do This
```rust
// ✗ String interpolation in SQL queries
sqlx::query(format!("SELECT * FROM users WHERE email = '{}'", email))

// ✗ Logging sensitive data
tracing::info!("User password: {}", password);

// ✗ Unencrypted private key storage
user.private_key // Should be encrypted

// ✗ Regular comparison for secrets
if password == stored_password // Use constant-time compare

// ✗ Weak cryptography
RSA 1024-bit // Use 4096-bit minimum
```

### Always Do This
```rust
// ✓ Parameterized queries
sqlx::query("SELECT * FROM users WHERE email = ?")
    .bind(email)
    .fetch_one(&pool)

// ✓ Structured logging
tracing::info!(user_id = %user_id, "User authenticated");

// ✓ Encrypt sensitive data
let encrypted = encrypt(&password_key, &private_key_bytes)?;

// ✓ Constant-time comparison
constant_time_compare(stored_hash.as_bytes(), computed_hash.as_bytes())

// ✓ Strong cryptography
RSA::generate(4096) // Always 4096-bit minimum
```

## Performance Optimization

### Database
- Use indexes on frequently queried columns
- Implement pagination (100 items/page)
- Cache blockchain chain (5 min TTL)
- Use prepared statements

### API
- Return only necessary fields
- Implement compression (gzip)
- Cache static responses
- Set appropriate HTTP cache headers

### Frontend
- Code splitting for React
- Lazy load components
- Cache API responses
- Minimize bundle size

## Deployment Checklist

Before production deployment:
- [ ] All tests passing
- [ ] Security audit completed
- [ ] Configuration hardened
- [ ] Database backups tested
- [ ] SSL/TLS certificates installed
- [ ] Monitoring set up
- [ ] Logging aggregation configured
- [ ] Team trained on system
- [ ] Incident response plan ready
- [ ] Disaster recovery plan tested

## Troubleshooting

### Backend won't compile
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build
```

### Tests failing
```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run specific test
cargo test test_name
```

### Database issues
```bash
# Reset migrations
rm backend/data/id_card_system.db
cargo sqlx migrate run
```

## Resources

- [Actix-web Docs](https://actix.rs/)
- [Tokio Docs](https://tokio.rs/)
- [SQLx Docs](https://github.com/launchbadge/sqlx)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
