# API Reference - ID Card System

## Base URL
```
http://localhost:8080/api/v1
```

## Authentication
All authenticated endpoints require a JWT token in the Authorization header:
```
Authorization: Bearer {token}
```

## Response Format
All responses are JSON and follow this format:
```json
{
  "data": {},          // Response payload
  "error": null,       // Error message (if any)
  "timestamp": "2024-01-15T10:30:45Z"
}
```

## Error Handling
Error responses include status code and error details:
```json
{
  "error": "Invalid credentials",
  "message": "Authentication failed",
  "status": 401
}
```

---

## Health Check

### GET /health
Check API server status.

**Response (200)**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:45Z",
  "version": "1.0.0"
}
```

---

## Authentication Endpoints

### POST /auth/register
Register a new student with credentials.

**Request Body**
```json
{
  "email": "student@university.edu",
  "password": "SecurePassword123!",
  "name": "John Doe",
  "institution": "Federal University of Technology Minna",
  "matric_number": "MTN/SCI/001/2023",
  "age": 20,
  "gender": "Male",
  "department": "Computer Science",
  "phone_contact": "+2341234567890"
}
```

**Response (201)**
```json
{
  "user_id": "user-123",
  "public_key": "-----BEGIN PUBLIC KEY-----\n...",
  "credential_id": "cred-456",
  "created_at": "2024-01-15T10:30:45Z"
}
```

**Errors**
- 400: Invalid input (missing or invalid fields)
- 409: User already exists
- 500: Internal server error

---

### POST /auth/login
Authenticate with email and password.

**Request Body**
```json
{
  "email": "student@university.edu",
  "password": "SecurePassword123!"
}
```

**Response (200)**
```json
{
  "user_id": "user-123",
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600
}
```

**Errors**
- 401: Invalid credentials
- 404: User not found
- 429: Too many login attempts

---

### POST /auth/verify
Verify digital signature for credential authentication.

**Request Body**
```json
{
  "public_key": "-----BEGIN PUBLIC KEY-----\n...",
  "message": "credential data",
  "signature": "signature_hex_string"
}
```

**Response (200)**
```json
{
  "valid": true,
  "user_id": "user-123"
}
```

**Errors**
- 400: Invalid input
- 401: Signature verification failed

---

### POST /auth/refresh
Refresh JWT token using refresh token.

**Request Body**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Response (200)**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600
}
```

**Errors**
- 401: Invalid or expired refresh token

---

## Credential Endpoints

### GET /credentials
List all credentials for authenticated user.

**Query Parameters**
- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 100, max: 1000)

**Response (200)**
```json
{
  "data": [
    {
      "id": "cred-123",
      "user_id": "user-456",
      "public_key_hash": "abc123...",
      "qr_hash": "def456...",
      "status": "active",
      "expires_at": "2025-01-15T10:30:45Z",
      "created_at": "2024-01-15T10:30:45Z"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 100
}
```

**Requires Authentication**: Yes

---

### GET /credentials/{id}
Get specific credential details.

**Path Parameters**
- `id`: Credential ID

**Response (200)**
```json
{
  "id": "cred-123",
  "user_id": "user-456",
  "public_key_hash": "abc123...",
  "qr_code": "data:image/png;base64,...",
  "status": "active",
  "created_at": "2024-01-15T10:30:45Z"
}
```

**Errors**
- 404: Credential not found
- 403: Not authorized

**Requires Authentication**: Yes

---

### POST /credentials/{id}/verify
Verify credential authenticity.

**Request Body**
```json
{
  "signature": "signature_hex_string"
}
```

**Response (200)**
```json
{
  "valid": true,
  "verified_at": "2024-01-15T10:30:45Z",
  "blockchain_confirmed": true
}
```

**Requires Authentication**: Yes

---

### POST /credentials/{id}/revoke
Revoke a credential (disable it).

**Response (200)**
```json
{
  "id": "cred-123",
  "status": "revoked",
  "revoked_at": "2024-01-15T10:30:45Z"
}
```

**Requires Authentication**: Yes

---

## Blockchain Endpoints

### GET /blockchain/chain
Get the entire blockchain.

**Query Parameters**
- `limit`: Maximum blocks to return (default: 100)

**Response (200)**
```json
{
  "data": [
    {
      "index": 0,
      "hash": "abc123...",
      "previous_hash": "0",
      "timestamp": "2024-01-15T10:30:45Z",
      "nonce": 1234,
      "difficulty": 2,
      "data": "{credential_data}"
    }
  ],
  "total": 100,
  "valid": true
}
```

---

### GET /blockchain/blocks/{id}
Get specific block details.

**Path Parameters**
- `id`: Block index

**Response (200)**
```json
{
  "index": 5,
  "hash": "abc123...",
  "previous_hash": "def456...",
  "timestamp": "2024-01-15T10:30:45Z",
  "nonce": 5678,
  "difficulty": 2,
  "data": "{credential_data}"
}
```

**Errors**
- 404: Block not found

---

### POST /blockchain/validate
Validate blockchain integrity.

**Response (200)**
```json
{
  "valid": true,
  "total_blocks": 100,
  "last_block_hash": "abc123..."
}
```

---

## QR Code Endpoints

### POST /qr/generate
Generate QR code for credential.

**Request Body**
```json
{
  "credential_id": "cred-123",
  "include_private_data": false
}
```

**Response (200)**
```json
{
  "qr_code_data": "credential_data_string",
  "qr_code_image": "data:image/png;base64,...",
  "encoding": "UTF-8"
}
```

**Requires Authentication**: Yes

---

### POST /qr/verify
Verify QR code and extract credential data.

**Request Body**
```json
{
  "qr_code_data": "credential_data_string"
}
```

**Response (200)**
```json
{
  "valid": true,
  "credential_id": "cred-123",
  "user_id": "user-456",
  "user_email": "student@university.edu"
}
```

---

## Payment Endpoints

### GET /payments
List user's payment transactions.

**Query Parameters**
- `status`: Filter by status (pending, completed, failed, revoked)
- `page`: Page number (default: 1)

**Response (200)**
```json
{
  "data": [
    {
      "id": "pay-123",
      "user_id": "user-456",
      "amount": "100.00",
      "asset_code": "USD",
      "recipient_address": "GABC123...",
      "status": "completed",
      "transaction_hash": "tx123...",
      "created_at": "2024-01-15T10:30:45Z",
      "completed_at": "2024-01-15T10:35:45Z"
    }
  ],
  "total": 1,
  "page": 1
}
```

**Requires Authentication**: Yes

---

### POST /payments
Create new payment transaction.

**Request Body**
```json
{
  "amount": "100.00",
  "asset_code": "USD",
  "recipient_address": "GABC123...",
  "merchant_id": "merchant-789",
  "description": "Tuition payment"
}
```

**Response (201)**
```json
{
  "id": "pay-123",
  "status": "pending",
  "amount": "100.00",
  "created_at": "2024-01-15T10:30:45Z"
}
```

**Errors**
- 400: Invalid amount or recipient
- 429: Payment limit exceeded

**Requires Authentication**: Yes

---

### GET /payments/{id}
Get payment details.

**Response (200)**
```json
{
  "id": "pay-123",
  "user_id": "user-456",
  "amount": "100.00",
  "asset_code": "USD",
  "status": "completed",
  "transaction_hash": "tx123...",
  "created_at": "2024-01-15T10:30:45Z"
}
```

**Requires Authentication**: Yes

---

### POST /payments/{id}/verify
Verify payment on Stellar network.

**Response (200)**
```json
{
  "valid": true,
  "transaction_hash": "tx123...",
  "ledger": 12345,
  "confirmed_at": "2024-01-15T10:32:45Z"
}
```

**Requires Authentication**: Yes

---

### POST /payments/merchant/qr
Generate merchant payment QR code.

**Request Body**
```json
{
  "amount": "100.00",
  "merchant_name": "University Bookstore",
  "merchant_id": "merchant-789"
}
```

**Response (200)**
```json
{
  "qr_code": "data:image/png;base64,...",
  "merchant_id": "merchant-789",
  "amount": "100.00"
}
```

---

## Biometric/WebAuthn Endpoints

### POST /biometric/register/start
Start biometric credential registration.

**Request Body**
```json
{
  "user_id": "user-123",
  "device_name": "iPhone 15 Pro"
}
```

**Response (200)**
```json
{
  "registration_id": "reg-123",
  "challenge": "base64_challenge_string"
}
```

**Requires Authentication**: Yes

---

### POST /biometric/register/complete
Complete biometric registration.

**Request Body**
```json
{
  "registration_id": "reg-123",
  "credential_data": "base64_credential_data"
}
```

**Response (200)**
```json
{
  "registered": true,
  "credential_id": "bio-123"
}
```

**Requires Authentication**: Yes

---

### POST /biometric/authenticate/start
Start biometric authentication.

**Request Body**
```json
{
  "user_id": "user-123"
}
```

**Response (200)**
```json
{
  "authentication_id": "auth-123",
  "challenge": "base64_challenge_string"
}
```

---

### POST /biometric/authenticate/complete
Complete biometric authentication.

**Request Body**
```json
{
  "authentication_id": "auth-123",
  "assertion_data": "base64_assertion_data"
}
```

**Response (200)**
```json
{
  "authenticated": true,
  "token": "eyJhbGciOiJIUzI1NiIs..."
}
```

---

## Admin Endpoints

### GET /admin/audit-log
Get audit log entries (admin only).

**Query Parameters**
- `user_id`: Filter by user ID
- `action`: Filter by action type
- `limit`: Number of entries (default: 100, max: 1000)
- `offset`: Pagination offset

**Response (200)**
```json
{
  "data": [
    {
      "id": "log-123",
      "user_id": "user-456",
      "action": "credential_verified",
      "resource_type": "credential",
      "resource_id": "cred-789",
      "details": "{...}",
      "ip_address": "192.168.1.1",
      "timestamp": "2024-01-15T10:30:45Z"
    }
  ],
  "total": 1000,
  "page": 1
}
```

**Requires Authentication**: Yes (Admin role)

---

### GET /admin/stats
Get system statistics (admin only).

**Response (200)**
```json
{
  "total_users": 1000,
  "total_credentials": 1000,
  "total_payments": 500,
  "blockchain_blocks": 2000,
  "avg_response_time_ms": 50,
  "active_sessions": 150
}
```

**Requires Authentication**: Yes (Admin role)

---

## Rate Limiting

All endpoints are rate limited:
- **Authentication endpoints**: 5 requests/minute per IP
- **API endpoints**: 100 requests/minute per user
- **Payment endpoints**: 50 requests/day per user

Rate limit headers in response:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1705320645
```

---

## HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | OK - Request successful |
| 201 | Created - Resource created |
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Authentication required |
| 403 | Forbidden - Not authorized for action |
| 404 | Not Found - Resource doesn't exist |
| 409 | Conflict - Resource already exists |
| 429 | Too Many Requests - Rate limited |
| 500 | Internal Server Error |
| 502 | Bad Gateway - Service unavailable |
| 503 | Service Unavailable - Maintenance |

---

## Examples

### Register and Login
```bash
# 1. Register
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "student@uni.edu",
    "password": "SecurePass123!",
    "name": "John Doe",
    "institution": "FUT Minna",
    "matric_number": "MTN/CS/001/2023",
    "age": 20,
    "gender": "Male",
    "department": "Computer Science",
    "phone_contact": "+2341234567890"
  }'

# 2. Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "student@uni.edu",
    "password": "SecurePass123!"
  }'

# 3. Store token and use in subsequent requests
TOKEN="eyJhbGciOiJIUzI1NiIs..."

# 4. List credentials
curl http://localhost:8080/api/v1/credentials \
  -H "Authorization: Bearer $TOKEN"
```

---

## Webhooks (Future)

Webhooks for payment confirmations and blockchain events (coming in v1.1):
- `payment.completed`
- `payment.failed`
- `credential.revoked`
- `blockchain.block_added`
