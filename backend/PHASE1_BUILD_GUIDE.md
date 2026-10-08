# Phase 1: Rust Backend Modernization - Build Guide

## Overview

Phase 1 implements the authentication layer with modular services architecture for the LEGACY student identity system.

## New Files Created

### Core Modules
- `src/jwt.rs` - JWT token generation and validation
- `src/audit.rs` - Audit event logging system
- `src/crypto/password.rs` - Password hashing with PBKDF2

### Service Layer (Modular Architecture)
- `src/services/mod.rs` - Service container and module exports
- `src/services/authentication.rs` - User registration and login
- `src/services/students.rs` - Student management (get, list, suspend, reactivate)
- `src/services/credentials.rs` - Credential lifecycle management
- `src/services/verification.rs` - Credential verification logic
- `src/services/cryptography.rs` - Cryptographic operations wrapper
- `src/services/qr.rs` - QR code generation and verification
- `src/services/payments.rs` - Payment request management
- `src/services/audit_service.rs` - Audit log retrieval and querying

### Middleware Updates
- `src/middleware.rs` - Added JwtMiddleware for route protection

### Modified Files
- `Cargo.toml` - Added aes-gcm, pbkdf2 dependencies
- `src/main.rs` - Added module declarations, wired JWT middleware to protected routes
- `src/models.rs` - Added sqlx::FromRow derive macros
- `src/crypto/mod.rs` - Exported password module

## Building Instructions

### Prerequisites
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env  # On Linux/macOS
# On Windows, Rust installer sets PATH automatically
```

### Build Steps
```bash
cd backend

# Check compilation without building
cargo check

# Run all tests including unit tests
cargo test

# Run linting checks
cargo clippy

# Build release binary
cargo build --release

# Run the server
cargo run
```

## API Endpoints (Phase 1)

### Authentication (Public)
- `POST /api/v1/auth/register` - Register new student
  - Request: `{ email, password, name, institution, matric_number, age, gender, department, phone_contact }`
  - Response: `{ user_id, public_key, credential_id, created_at }`
  
- `POST /api/v1/auth/login` - Login user
  - Request: `{ email, password }`
  - Response: `{ user_id, token, refresh_token, expires_in }`

### Protected Routes (Require JWT Token)
All requests must include: `Authorization: Bearer <token>`

- `GET /api/v1/credentials` - List user credentials
- `GET /api/v1/credentials/{id}` - Get credential details
- `POST /api/v1/credentials` - Create credential (stub)
- `POST /api/v1/credentials/{id}/verify` - Verify credential (stub)
- `POST /api/v1/credentials/{id}/revoke` - Revoke credential (stub)

### Public Routes
- `GET /health` - Health check
- `GET /api/v1/blockchain/chain` - Get blockchain (stub)
- `POST /api/v1/blockchain/validate` - Validate blockchain (stub)
- `POST /api/v1/qr/verify` - Verify QR code (stub)

## Service Architecture

### Layered Design

```
HTTP Handlers
    ↓
Services (Business Logic)
    ↓
Database Layer (SQLx ORM)
    ↓
SQLite/PostgreSQL
```

### Services Overview

#### AuthenticationService
- `register(RegisterRequest) → user_id`
- `login(LoginRequest) → (user_id, token)`
- `validate_token(token) → JwtClaims`

#### StudentService
- `get_by_id(student_id) → Option<User>`
- `get_by_email(email) → Option<User>`
- `list_all(limit, offset) → Vec<User>`
- `suspend(student_id) → ()`
- `reactivate(student_id) → ()`

#### CredentialService
- `create(...) → Credential`
- `get_by_id(credential_id) → Option<Credential>`
- `list_by_user(user_id) → Vec<Credential>`
- `revoke(credential_id) → ()`
- `expire(credential_id) → ()`
- `is_valid(credential_id) → bool`

#### VerificationService
- `verify_credential(credential_id) → VerificationResult`
- `verify_signature(credential_id, signature) → bool`
- `verify_qr_hash(credential_id, hash) → bool`

#### CryptographyService
- `generate_rsa_keypair() → (String, String)`
- `hash_sha256(data) → String`
- `hash_password(password) → (String, String)`
- `verify_password(password, hash, salt) → bool`
- `sign_data(message, private_key) → String`
- `verify_signature(message, signature, public_key) → bool`

#### QrService
- `generate(data) → Vec<u8>`
- `hash_qr(data) → String`
- `verify_hash(data, expected_hash) → bool`

#### PaymentService
- `create_payment_request(...) → ()`
- `get_payment_status(payment_id) → Option<String>`
- `update_payment_status(payment_id, status) → ()`
- `list_user_payments(user_id) → Vec<PaymentInfo>`

#### AuditService
- `get_user_logs(user_id, limit) → Vec<AuditLogEntry>`
- `get_resource_logs(resource_type, resource_id, limit) → Vec<AuditLogEntry>`
- `get_all_logs(limit, offset) → Vec<AuditLogEntry>`
- `count_events(action) → i64`

## Cryptography Stack

### Implemented
- **Password Hashing**: PBKDF2-SHA256 (100,000 iterations)
- **Key Derivation**: PBKDF2-SHA256
- **SHA-256**: SHA2-based hashing for credentials and QR codes
- **RSA 4096**: User keypair generation
- **EdDSA**: Digital signatures (available, not yet integrated)
- **AES-256-GCM**: Private key encryption (available for future use)

### Password Requirements
- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one digit
- At least one special character

## Middleware

### JWT Middleware
Applied to protected routes:
- `/api/v1/credentials/*`
- `/api/v1/payments/*`
- `/api/v1/biometric/*`
- `/api/v1/admin/*`

Extracts `Authorization: Bearer <token>` header, validates token, stores claims in request extensions.

### CORS Middleware
Basic CORS support for cross-origin requests.

## Audit Logging

### Logged Events
- `STUDENT_CREATED` - User registered
- `STUDENT_SUSPENDED` - Student suspended
- `LOGIN_SUCCESS` - Successful login
- `LOGIN_FAILURE` - Failed login attempt
- `CREDENTIAL_ISSUED` - Credential created
- `CREDENTIAL_VERIFIED` - Credential verified
- `CREDENTIAL_REVOKED` - Credential revoked
- `CREDENTIAL_EXPIRED` - Credential expired
- `BIOMETRIC_AUTH_SUCCESS` - Biometric auth succeeded
- `BIOMETRIC_AUTH_FAILURE` - Biometric auth failed
- `PAYMENT_REQUESTED` - Payment created
- `PAYMENT_AUTHORIZED` - Payment authorized
- `PAYMENT_SUBMITTED` - Payment submitted
- `PAYMENT_CONFIRMED` - Payment confirmed
- `PAYMENT_REJECTED` - Payment rejected

## Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test crypto::tests
cargo test jwt::tests
cargo test audit::tests
```

### Integration Tests (TODO - Phase 2)
- User registration flow
- Authentication flow
- Credential lifecycle
- Payment flow
- Biometric authentication

## Configuration

### Environment Variables
```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
DATABASE_URL=sqlite://data/id_card_system.db
JWT_SECRET=your-secret-at-least-32-characters-long
JWT_EXPIRY_SECS=3600
PBKDF2_ITERATIONS=100000
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
ENABLE_BIOMETRIC_AUTH=true
ENABLE_PAYMENTS=true
```

## Error Handling

All errors return structured JSON responses with appropriate HTTP status codes:
- `400 Bad Request` - Validation errors
- `401 Unauthorized` - Authentication failures
- `403 Forbidden` - Authorization failures
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource already exists
- `500 Internal Server Error` - Server errors

## Next Steps (Phase 2)

1. Implement credential creation with QR generation
2. Implement credential verification with signature validation
3. Integrate blockchain records for credentials
4. Implement payment flow with Stellar integration
5. Implement WebAuthn/biometric authentication
6. Complete RBAC enforcement
7. Add comprehensive integration tests

## Architecture Benefits

✅ **Modularity** - Each service handles one domain  
✅ **Testability** - Services can be tested independently  
✅ **Reusability** - Services can be used across handlers  
✅ **Security** - Cryptographic operations centralized  
✅ **Maintainability** - Clear separation of concerns  
✅ **Scalability** - Easy to add new services  

## Troubleshooting

### Build Errors

**Error: "cannot find crate `aes_gcm`"**
- Solution: Ensure `Cargo.toml` has `aes-gcm = "0.10"`
- Run `cargo update`

**Error: "cannot find function `hash_password`"**
- Solution: Ensure `src/crypto/password.rs` is created
- Verify `src/crypto/mod.rs` exports the password module

**Error: "cannot find type `User`"**
- Solution: Ensure models have `sqlx::FromRow` derive macro
- Check imports in services

### Runtime Errors

**Error: "no such table: users"**
- Solution: Run migrations: `sqlx migrate run`
- Verify database file exists at `data/id_card_system.db`

**Error: "Invalid Authorization header format"**
- Solution: Send JWT with format: `Authorization: Bearer <token>`
- Ensure token is properly formatted

## Performance Considerations

- JWT validation is fast (O(1))
- Password hashing is intentionally slow (100,000 iterations PBKDF2)
- Database queries use indexes for common lookups
- Connection pooling via SqlitePool

## Security Considerations

✅ Passwords never stored plaintext - hashed with PBKDF2  
✅ JWT tokens validate signature - tamper-proof  
✅ SQL injection prevention - parameterized queries via SQLx  
✅ Password strength validation - enforced requirements  
✅ Audit logging - all security events recorded  
✅ HTTP-only (configured for TLS in production)  

---

**Phase 1 Status**: ✅ COMPLETE (Authentication Layer)  
**Ready for Phase 2**: Credential Lifecycle + Blockchain Integration
