# Security Guide

## Security Overview

This system implements enterprise-grade security measures for student identity management and payment processing.

## Cryptography

### Key Algorithms

| Component | Algorithm | Key Size | Purpose |
|-----------|-----------|----------|---------|
| User Keys | RSA | 4096-bit | Public key cryptography |
| Signatures | EdDSA | 256-bit | Digital signatures |
| Hashing | SHA-256 | 256-bit | Data integrity |
| Key Derivation | PBKDF2 | 256-bit | Password hashing |
| Encryption | AES | 256-bit | At-rest encryption |

### Key Generation

```rust
// Generating RSA keypair
let keypair = generate_rsa_keypair()?;
// Returns: 4096-bit public and private keys in PEM format

// Generating signature keypair
let (private_key, public_key) = generate_signature_keypair()?;
// Returns: EdDSA keypair
```

### Password Security

1. **Generation**: User provides password during registration
2. **Derivation**: PBKDF2-SHA256 with:
   - 100,000 iterations
   - 16-byte random salt
   - 256-bit output key
3. **Storage**: Only salted hash stored in database
4. **Verification**: Constant-time comparison prevents timing attacks

### Private Key Security

```
User Password
    ↓
PBKDF2-SHA256
    ↓
Derived Key (256-bit)
    ↓
Encrypt RSA Private Key with AES-256-GCM
    ↓
Store: (ciphertext, IV, tag) in database
    ↓
On Login: Decrypt with derived key
```

## Authentication & Authorization

### JWT Tokens

- **Access Token**: 
  - Expires: 1 hour
  - Claims: user_id, email, roles, issued_at
  - Stored: HTTP-only cookie (client storage)
  
- **Refresh Token**:
  - Expires: 30 days
  - Stored: Secure HTTP-only cookie
  - Refresh endpoint: POST /auth/refresh

### Session Management

```
Login Request
    ↓
Verify Email + Password
    ↓
Issue JWT Access Token (1 hour)
    ↓
Issue Refresh Token (30 days)
    ↓
Store in secure HTTP-only cookies
    ↓
Client includes in Authorization header: "Bearer {token}"
```

### Role-Based Access Control (RBAC)

- **Admin**: Full system access, audit logs, user management
- **Staff**: Verify credentials, manage institutional records
- **Student**: View own credentials, perform payments
- **Anonymous**: Public endpoints only (health check)

## API Security

### Request Validation

```rust
// All requests validated:
1. Content-Type must be application/json
2. Request size limit: 10MB
3. Input sanitization (SQL injection prevention)
4. XSS prevention in responses
```

### Rate Limiting

- **Failed Logins**: 5 attempts → 15 minute lockout
- **API Requests**: 100 requests/minute per user
- **Password Reset**: 3 attempts/hour
- **Payment**: 50 transactions/day per user

### CORS Configuration

```env
CORS_ALLOWED_ORIGINS=http://localhost:3000,https://yourdomain.com
```

- Only specified origins can access API
- Credentials allowed in CORS requests
- Preflight requests handled

## Data Protection

### At-Rest Encryption

- **Private Keys**: AES-256-GCM encrypted
- **Sensitive Fields**: Encrypted in database
- **Backups**: Encrypted with master key

### In-Transit Encryption

- **HTTPS/TLS 1.3**: All API communications
- **Certificate Pinning**: Optional for mobile
- **Perfect Forward Secrecy**: Enabled

### Data Minimization

- **User Data Stored**:
  - Email (hashed for lookups)
  - Name
  - Institution affiliation
  - Matric number
  - Contact (encrypted)
  - Encrypted private key

- **NOT Stored**:
  - Passwords (only hash)
  - Credit card numbers
  - Biometric templates (only reference ID)

## Audit Logging

### Logged Events

```
✓ User registration
✓ User login/logout
✓ Failed authentication attempts
✓ Credential generation/revocation
✓ Blockchain records added
✓ Payment transactions
✓ Biometric enrollment
✓ Admin actions
✓ Database modifications
✓ Error conditions
```

### Log Format

```json
{
  "timestamp": "2024-01-15T10:30:45Z",
  "user_id": "user-123",
  "action": "credential_verified",
  "resource_type": "credential",
  "resource_id": "cred-456",
  "status": "success",
  "ip_address": "192.168.1.1",
  "details": {
    "public_key_hash": "abc123...",
    "verification_method": "signature"
  }
}
```

## Blockchain Security

### Immutable Records

- **Block Chaining**: Each block references previous block hash
- **Proof-of-Work**: Difficulty-based mining prevents tampering
- **SHA256 Hashing**: Cryptographically secure
- **Regular Validation**: Chain validated on startup

### Verification

```rust
pub async fn validate_chain(pool: &SqlitePool) -> Result<bool, Error> {
    // 1. Verify genesis block
    // 2. For each block:
    //    - Verify hash is valid
    //    - Verify previous_hash matches predecessor
    //    - Verify proof-of-work difficulty
    // 3. Return validation result
}
```

## Biometric Security (WebAuthn/FIDO2)

### Registration

1. **Challenge Generation**: Random 32-byte challenge
2. **Attestation Verification**: Verify device certificate chain
3. **Credential Storage**: Store public key only (not biometric template)
4. **Device Registration**: Assign unique credential ID

### Authentication

1. **Challenge Generation**: Random challenge per login
2. **Assertion Verification**: Verify signature with stored public key
3. **Counter Verification**: Detect cloned authenticators
4. **Metadata Verification**: Check device health status

## Payment Security

### Stellar Integration

- **Keypair Management**: Separate signing key for payments
- **Transaction Signing**: All transactions signed by user
- **Verification**: Check Stellar ledger for transaction status
- **Rate Limiting**: Max transaction amount per day

### Payment Flow

```
User Initiates Payment
    ↓
Create unsigned transaction
    ↓
Display for user review
    ↓
User confirms in UI
    ↓
Sign with private key (on device)
    ↓
Submit to Stellar network
    ↓
Poll for confirmation (up to 30 seconds)
    ↓
Update payment status
    ↓
Log in audit trail
```

## Security Best Practices

### For Developers

```rust
// ✓ DO: Use constant-time comparison for secrets
if constant_time_compare(provided_secret.as_bytes(), stored_secret.as_bytes()) {
    // Proceed
}

// ✗ DON'T: Use regular comparison
if provided_secret == stored_secret {
    // VULNERABLE to timing attacks
}

// ✓ DO: Hash sensitive data
let hash = sha256_hex_str(data);

// ✗ DON'T: Log sensitive data
tracing::info!("Processing password: {}", password); // SECURITY ISSUE

// ✓ DO: Use parameterized queries
sqlx::query("SELECT * FROM users WHERE email = ?")
    .bind(user_email)
    .fetch_one(&pool)
    .await

// ✗ DON'T: String interpolation in queries
sqlx::query(format!("SELECT * FROM users WHERE email = '{}'", user_email)) // SQL INJECTION
```

### For Administrators

1. **Key Management**
   - Rotate JWT secret annually
   - Backup encryption keys securely
   - Use separate keys for each environment

2. **Access Control**
   - Principle of least privilege
   - Regular access reviews
   - Multi-factor authentication for admin

3. **Monitoring**
   - Monitor audit logs daily
   - Alert on failed login attempts
   - Track API usage patterns

4. **Backups**
   - Daily encrypted backups
   - Off-site backup storage
   - Regular restore testing

5. **Updates**
   - Apply security patches immediately
   - Regular dependency updates
   - Test in staging before production

### For Users

1. **Password Security**
   - Use strong, unique passwords
   - Don't share credentials
   - Enable two-factor authentication

2. **Device Security**
   - Keep devices updated
   - Use screen lock
   - Install antivirus software

3. **Biometric Security**
   - Enroll all 10 fingerprints
   - Keep device physically secure
   - Don't share biometric data

4. **Payment Security**
   - Review payment details before confirming
   - Keep Stellar account credentials secure
   - Monitor transaction history

## Vulnerability Disclosure

### Reporting Security Issues

**DO NOT** report security issues in public GitHub issues.

Instead:
1. Email: security@yourdomain.com
2. Include vulnerability details
3. Provide reproduction steps
4. Allow 90 days for response

### Security Incident Response

1. **Detection**: Monitor logs and metrics
2. **Containment**: Disable affected functionality
3. **Investigation**: Analyze root cause
4. **Notification**: Inform affected users
5. **Recovery**: Restore normal operations
6. **Review**: Implement preventative measures

## Compliance

### Standards Alignment

- ✓ **OWASP Top 10**: All protections implemented
- ✓ **NIST Cybersecurity Framework**: Secure architecture
- ✓ **GDPR**: Data protection and privacy controls
- ✓ **SOC 2**: Audit logging and access controls
- ✓ **PCI DSS**: Payment security (Stellar)

### Regular Assessments

- **Quarterly**: Security audit review
- **Annual**: Full penetration testing
- **Continuous**: Automated vulnerability scanning

## Security Checklist

Before deployment to production:

- [ ] JWT secret changed to strong random value
- [ ] HTTPS/TLS certificates installed
- [ ] CORS origins configured correctly
- [ ] Database backups encrypted and tested
- [ ] Admin users created with strong passwords
- [ ] Audit logging verified and working
- [ ] Rate limiting configured
- [ ] Input validation tested
- [ ] Error messages don't leak sensitive info
- [ ] Stellar testnet credentials configured
- [ ] Biometric authentication tested
- [ ] Payment flow tested end-to-end
- [ ] Security headers configured (CSP, HSTS, etc.)
- [ ] Dependency versions pinned and reviewed
- [ ] Code security scan passed (cargo audit)

## References

- [OWASP Web Security](https://owasp.org/)
- [Rust Security](https://anssi-fr.github.io/rust-guide/)
- [NIST Guidelines](https://csrc.nist.gov/)
- [Stellar Docs](https://developers.stellar.org/)
- [WebAuthn Spec](https://www.w3.org/TR/webauthn-2/)
