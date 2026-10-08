# System Architecture - Student Identity Card Security System

## Overview

This is a modern, production-ready authentication and credential verification system combining:
- **RSA 4096-bit cryptography** for key management
- **EdDSA digital signatures** for authentication
- **SHA256 hashing** for data integrity
- **Blockchain verification** for credential history
- **WebAuthn/FIDO2** for biometric authentication
- **Stellar payments** for transaction support
- **Comprehensive audit logging** for compliance

## System Architecture

```
┌─────────────────────────────────────────────────┐
│           Frontend Layer (Client)                │
├──────────────────┬──────────────────┬───────────┤
│  React SPA       │  Streamlit MVP   │ Mobile    │
│  (Production)    │  (Development)   │ Apps      │
└────────┬─────────┴────────┬─────────┴───────────┘
         │                  │
    HTTPS/REST             HTTPS/REST
         │                  │
┌────────▼──────────────────▼─────────────────────┐
│         API Gateway / Load Balancer             │
│  ├─ Authentication Middleware                   │
│  ├─ CORS Configuration                          │
│  ├─ Rate Limiting                               │
│  └─ Request Validation                          │
└──────────────────┬────────────────────────────┘
                   │
┌──────────────────▼────────────────────────────┐
│        Rust Backend (Actix-Web)               │
├──────────────────────────────────────────────┤
│ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│ │   Auth   │ │Credential│ │Blockchain│       │
│ │ Service  │ │ Service  │ │ Service  │       │
│ └──────────┘ └──────────┘ └──────────┘       │
│                                              │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│ │ Payment  │ │Biometric │ │  QR      │       │
│ │ Service  │ │ Service  │ │ Service  │       │
│ └──────────┘ └──────────┘ └──────────┘       │
└──────┬──────────────────────────────────────┘
       │
   ┌───┴────┬─────────┬──────────┬────────────┐
   │        │         │          │            │
   v        v         v          v            v
┌──────┐ ┌────────┐ ┌──────────┐ ┌─────────┐ ┌──────────┐
│SQLite│ │Crypto  │ │Blockchain│ │Stellar  │ │ Audit    │
│DB    │ │Module  │ │Storage   │ │Network  │ │ Logger   │
└──────┘ └────────┘ └──────────┘ └─────────┘ └──────────┘
```

## Layer Breakdown

### 1. Frontend Layer

#### React SPA (Production)
- Modern TypeScript-based single-page application
- WebAuthn integration for biometric authentication
- Real-time QR code scanning/generation
- Payment authorization UI
- Real-time audit log viewer

#### Streamlit MVP (Development)
- Rapid prototyping interface
- All core features accessible
- WebRTC-based video QR scanning
- API testing capabilities

### 2. API Gateway Layer
- Request validation and sanitization
- Authentication token verification
- CORS handling
- Rate limiting per user
- Request/response logging

### 3. Rust Backend Services

#### Authentication Service (`auth`)
```
POST /auth/register     - Register new student with credentials
POST /auth/login        - Login and receive JWT token
POST /auth/verify       - Verify digital signature
POST /auth/refresh      - Refresh JWT token
```

#### Credential Service (`credentials`)
```
GET  /credentials       - List user credentials
GET  /credentials/{id}  - Get credential details
POST /credentials       - Create new credential
POST /credentials/{id}/verify  - Verify credential
POST /credentials/{id}/revoke   - Revoke credential
```

#### Blockchain Service (`blockchain`)
```
GET  /blockchain/chain       - Get full blockchain
GET  /blockchain/blocks/{id} - Get block details
POST /blockchain/validate    - Validate chain integrity
```

#### QR Service (`qr`)
```
POST /qr/generate  - Generate QR code
POST /qr/verify    - Verify QR code data
```

#### Payment Service (`payments`)
```
GET  /payments                - List user payments
POST /payments                - Create new payment
GET  /payments/{id}           - Get payment details
POST /payments/{id}/verify    - Verify payment
POST /payments/merchant/qr    - Generate merchant QR
```

#### Biometric Service (`biometric`)
```
POST /biometric/register/start      - Start biometric registration
POST /biometric/register/complete   - Complete registration
POST /biometric/authenticate/start  - Start authentication
POST /biometric/authenticate/complete - Complete authentication
```

#### Admin Service (`admin`)
```
GET /admin/audit-log  - View audit logs
GET /admin/stats      - System statistics
GET /admin/users      - List all users
```

### 4. Data Storage Layer

#### SQLite Database
- Users table (identity information)
- Credentials table (public keys, QR codes)
- Blockchain table (immutable transaction history)
- Payments table (Stellar transactions)
- Biometric credentials table (WebAuthn credentials)
- Audit logs table (security events)

#### Cryptographic Module
- RSA 4096-bit key generation and storage
- PBKDF2-SHA256 password key derivation
- EdDSA signature generation and verification
- SHA256 hash computation

#### Blockchain Storage
- Block chain with proof-of-work
- SHA256-based hashing
- Nonce-based difficulty
- Transaction verification

#### Stellar Integration
- Testnet network connectivity
- Transaction signing and submission
- Payment status tracking
- Asset management

#### Audit Logger
- All authentication events
- Credential lifecycle changes
- Payment transactions
- Error conditions
- Admin actions

## Security Architecture

### Cryptographic Strategy
1. **User Registration**
   - Generate RSA 4096-bit keypair
   - Derive password key using PBKDF2-SHA256
   - Encrypt private key with derived key
   - Store encrypted private key in database

2. **Authentication**
   - User provides email + password
   - Verify password against stored hash
   - Issue JWT token (short-lived, 1 hour)
   - Issue refresh token (long-lived, 30 days)

3. **Digital Signatures**
   - Create EdDSA signing keypair for each user
   - Sign identity data with private key
   - Store signature in credential record
   - Verify signature during authentication

4. **Blockchain Integrity**
   - Each credential creates blockchain record
   - Hash credential data with SHA256
   - Link blocks chronologically
   - Periodically validate chain integrity

### Key Management

```
Plaintext Password
    ↓
PBKDF2-SHA256 (100,000 iterations)
    ↓
Derived Key (256 bits)
    ↓
Encrypt RSA Private Key (AES-256-GCM)
    ↓
Store Encrypted Private Key in DB
    ↓
On Login: Decrypt with Derived Key
```

### Rate Limiting
- 5 failed login attempts → 15 minute lockout
- 100 API requests per minute per user
- 1000 audit log entries per day per user

### Session Management
- JWT tokens expire after 1 hour
- Refresh tokens expire after 30 days
- Token revocation on logout
- CORS prevents cross-domain requests

## Data Flow Examples

### User Registration Flow
```
1. User submits: email, password, personal info
2. Backend validates input
3. Generate RSA 4096 keypair
4. Hash password with PBKDF2
5. Store user record with encrypted private key
6. Generate EdDSA signature for identity
7. Create credential record with signature
8. Add credential to blockchain block
9. Mine block (PoW)
10. Issue JWT token
11. Return registration response
```

### Authentication Flow
```
1. User submits: email, password
2. Lookup user by email
3. Verify password hash
4. Generate JWT token
5. Generate refresh token
6. Return tokens to client
7. Client stores tokens in secure storage
8. Subsequent requests include JWT in Authorization header
```

### Payment Flow
```
1. User initiates payment with amount + recipient
2. Create payment record (pending status)
3. Generate Stellar transaction
4. User reviews and confirms in UI
5. Sign transaction with user's Stellar key
6. Submit to Stellar testnet
7. Poll for confirmation
8. Update payment status (completed)
9. Log transaction in audit trail
10. Return transaction hash to client
```

### Biometric Registration Flow
```
1. User initiates biometric registration
2. Server generates WebAuthn registration challenge
3. Client prompts biometric input
4. Device creates credential
5. Client sends attestation response
6. Server verifies credential
7. Store credential in database
8. Confirm registration complete
```

## Performance Considerations

### Database Optimization
- Indexed queries on: email, user_id, credential_id, transaction_hash
- Pagination on large result sets (100 items per page)
- Connection pooling (10 concurrent connections)
- Transaction batching for blockchain records

### Caching Strategy
- Redis cache for user sessions (1 hour TTL)
- Cache blockchain chain (5 minute TTL)
- Cache user credentials (1 hour TTL)
- Cache payment status (15 minute TTL)

### API Response Times
- Authentication endpoints: < 100ms
- Credential verification: < 200ms
- Blockchain operations: < 500ms
- Payment operations: < 2s (Stellar network dependent)

## Deployment Architecture

### Docker Containers
```
id-card-backend:latest    - Rust API server
id-card-frontend:latest   - React SPA (Nginx)
postgres:15-alpine        - Database (optional)
redis:7-alpine           - Cache layer
```

### Production Deployment
```
Load Balancer (SSL/TLS)
    ↓
┌─────────────────────────────┐
│   Kubernetes Cluster        │
│ ┌─────────────┐             │
│ │ API Pod 1-N │ (Rust)      │
│ ├─────────────┤             │
│ │ Cache Pod   │ (Redis)     │
│ ├─────────────┤             │
│ │ DB Pod      │ (SQLite)    │
│ └─────────────┘             │
└─────────────────────────────┘
    ↓
External Services:
  - Stellar Horizon API
  - Email Service
  - Log Aggregation
```

## Compliance & Audit

- ✅ Full audit trail of all operations
- ✅ User identity verification
- ✅ Credential lifecycle tracking
- ✅ Payment transaction logging
- ✅ Admin action logging
- ✅ GDPR-compliant data storage
- ✅ SOC 2 ready architecture
