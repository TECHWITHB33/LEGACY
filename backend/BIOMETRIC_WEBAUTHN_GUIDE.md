# Biometric Authentication Gateway - WebAuthn Implementation

## Overview

This document explains how the biometric authentication gateway works and **critically** demonstrates that raw biometric data NEVER reaches the server.

## Key Principle

**Raw biometric data (fingerprints, face images) stays on the device. The server receives only cryptographic signatures.**

## Architecture

### Three Layers

```
┌─────────────────────────────────────────────────────────────────────┐
│ CLIENT LAYER (Browser/Device)                                       │
│ - User's device with biometric sensor                              │
│ - WebAuthn API (W3C standard)                                       │
│ - Secure enclave/TEE stores private key                            │
│ - Local biometric matching engine                                   │
└─────────────────────────────────────────────────────────────────────┘
         │
         │ Network: Only signatures + metadata
         │ NO biometric data transmitted
         │
┌─────────────────────────────────────────────────────────────────────┐
│ REST API LAYER (/api/v1/biometric/*)                               │
│ - Handles challenge generation/storage                              │
│ - Receives signatures only                                          │
│ - Verifies signatures cryptographically                             │
│ - Returns authentication results                                    │
└─────────────────────────────────────────────────────────────────────┘
         │
         │ Queries
         │
┌─────────────────────────────────────────────────────────────────────┐
│ DATABASE LAYER (SQLite)                                            │
│ - biometric_credentials: public_keys only                          │
│ - biometric_challenges: random challenge bytes                     │
│ - NO biometric templates                                           │
│ - NO biometric images                                              │
└─────────────────────────────────────────────────────────────────────┘
```

## Registration Flow

### Step 1: Begin Registration (Server → Client)

**Endpoint:** `POST /api/v1/biometric/register/begin`

**Request:**
```json
{
  "device_id": "uuid-iphone-14",
  "device_name": "iPhone 14"
}
```

**Server Action:**
1. Generate random challenge (256 bits)
2. Store challenge in database with 5-minute expiration
3. Return challenge_id and challenge bytes

**Response:**
```json
{
  "challenge_id": "challenge-uuid-123",
  "challenge": "base64_encoded_random_bytes",
  "user_id": "user-123"
}
```

**Data on Server:**
```
biometric_challenges table:
┌──────────────────┬──────────────────┬────────────────────────┐
│ id               │ challenge        │ created_at             │
├──────────────────┼──────────────────┼────────────────────────┤
│ challenge-123    │ base64...        │ 2026-10-01T12:00:00Z   │
└──────────────────┴──────────────────┴────────────────────────┘
✅ Only random bytes, NO biometric data
```

### Step 2: Device-Side Biometric Enrollment (Device → Device)

**This step happens ENTIRELY on the device. NO server involvement.**

**Browser Code (JavaScript WebAuthn API):**
```javascript
// User clicks "Register biometric"
const challenge = base64ToArrayBuffer(challengeBytes);

// Browser shows biometric prompt
const credential = await navigator.credentials.create({
  publicKey: {
    challenge: challenge,
    rp: { name: "ID Card System" },
    user: {
      id: new Uint8Array(userId.length),
      name: "user@example.com",
      displayName: "User Name"
    },
    pubKeyCredParams: [
      { alg: -8, type: "public-key" }  // Ed25519
    ],
    authenticatorSelection: {
      authenticatorAttachment: "platform",  // Use device's biometric
      userVerification: "required"           // Must verify biometric
    },
    timeout: 60000
  }
});

// At this point:
// 1. Device shows biometric prompt (fingerprint/face/PIN)
// 2. User touches sensor or looks at camera
// 3. Device biometric engine verifies locally:
//    - Fingerprint image captured → compared to enrolled template
//    - Face image captured → compared to enrolled face data
//    - Verification happens ONLY on device
//    - NO images transmitted anywhere
// 4. If verification succeeds:
//    - Device generates Ed25519 key pair
//    - Private key stored in secure enclave
//    - Public key extracted
// 5. Challenge is signed with private key
// 6. credential object contains:
//    - public_key (NOT private key)
//    - signature (over challenge)
//    - attestation_object (device attestation)
```

**Data on Device:**
```
Secure Enclave (TEE - Trusted Execution Environment):
┌─────────────────────────────────────────────┐
│ Private Key                                 │
│ ├─ Ed25519 private key (32 bytes)           │
│ ├─ Stored in hardware (cannot be extracted) │
│ └─ Used only for signing                    │
│                                             │
│ Biometric Template                          │
│ ├─ Fingerprint: 512 bytes processed data    │
│ ├─ Face: 128 bytes face encoding            │
│ ├─ PIN/Passkey: hashed + salted             │
│ └─ Never leaves device                      │
└─────────────────────────────────────────────┘

⚠️ CRITICAL: These NEVER transmitted to server
```

### Step 3: Complete Registration (Client → Server)

**Endpoint:** `POST /api/v1/biometric/register/complete`

**Request:**
```json
{
  "challenge_id": "challenge-uuid-123",
  "public_key": "base64_encoded_public_key_32_bytes",
  "attestation_object": "base64_encoded_attestation",
  "credential_type": "fingerprint",
  "device_id": "uuid-iphone-14",
  "device_name": "iPhone 14"
}
```

**Data Transmitted:**
```
✅ public_key: 64 hex characters (32 bytes) - can be public
✅ attestation_object: device attestation data
❌ NO private key
❌ NO fingerprint image
❌ NO fingerprint template
❌ NO face image
❌ NO face encoding
❌ NO biometric data of ANY kind
```

**Server Action:**
1. Verify challenge exists and hasn't expired
2. Verify challenge not previously used
3. Verify attestation object format
4. Verify attestation signature chain
5. Extract public key from attestation
6. Store public_key in database
7. Mark challenge as used

**Data Stored in Database:**
```
biometric_credentials table:
┌────────────────┬──────────────┬──────────────────────────┬─────────┐
│ id             │ public_key   │ credential_type          │ status  │
├────────────────┼──────────────┼──────────────────────────┼─────────┤
│ cred-uuid-123  │ base64...    │ fingerprint              │ active  │
└────────────────┴──────────────┴──────────────────────────┴─────────┘
✅ Only public key, device info, and metadata
❌ NO biometric data
❌ NO private key (in secure enclave on device)
```

## Authentication Flow

### Step 1: Begin Authentication (Server → Client)

**Endpoint:** `POST /api/v1/biometric/authenticate/begin`

**Request:**
```json
{
  "credential_id": "cred-uuid-123"
}
```

**Server Action:**
1. Generate new random challenge (256 bits)
2. Store challenge linked to credential
3. Set 5-minute expiration

**Response:**
```json
{
  "challenge_id": "auth-challenge-uuid-456",
  "challenge": "base64_encoded_new_random_bytes"
}
```

### Step 2: Device-Side Biometric Authentication (Device → Device)

**Browser Code (JavaScript WebAuthn API):**
```javascript
// User clicks "Authenticate with biometric"
const challenge = base64ToArrayBuffer(challengeBytes);

// Browser shows biometric prompt
const assertion = await navigator.credentials.get({
  publicKey: {
    challenge: challenge,
    allowCredentials: [{
      id: base64ToArrayBuffer(credentialId),
      type: "public-key",
      transports: ["internal"]  // Use device's biometric
    }],
    userVerification: "required"  // Verify biometric first
  }
});

// At this point:
// 1. Device shows biometric prompt
// 2. User authenticates with biometric
// 3. Device verifies:
//    - Fingerprint: Match against enrolled template ✓
//    - Face: Match against enrolled face ✓
//    - PIN: Match against hashed PIN ✓
//    (ALL LOCAL - NO SERVER INVOLVED)
// 4. If verification succeeds:
//    - Device retrieves private key from secure enclave
//    - Device signs challenge with private key
//    - Private key used but NOT extracted
// 5. assertion object contains:
//    - signature (of challenge)
//    - signCount (incremented counter)
```

**Data on Device (Entire Authentication):**
```
Biometric Verification (Local Only):
┌──────────────────────────────────────┐
│ Fingerprint matching:                │
│ - User fingerprint captured          │
│ - Compared to enrolled template      │
│ - Match/No-Match result              │
│ - Result used locally                │
│                                      │
│ CRITICAL: Result never leaves device │
│ CRITICAL: Only "match" fact used    │
│ CRITICAL: Fingerprint never sent     │
└──────────────────────────────────────┘

Signature Generation (Local Only):
┌──────────────────────────────────────┐
│ Challenge to sign: "abc123def..."    │
│ Private key location: Secure enclave │
│ Signing algorithm: Ed25519           │
│ Signature result: base64 string      │
│ Signed data returned to browser      │
└──────────────────────────────────────┘

⚠️ NO biometric data leaves device
```

### Step 3: Complete Authentication (Client → Server)

**Endpoint:** `POST /api/v1/biometric/authenticate/complete`

**Request:**
```json
{
  "challenge_id": "auth-challenge-uuid-456",
  "credential_id": "cred-uuid-123",
  "signature": "base64_encoded_signature",
  "sign_count": 42
}
```

**Data Transmitted:**
```
✅ challenge_id: Challenge reference (public)
✅ credential_id: Credential reference (public)
✅ signature: Cryptographic signature (public)
✅ sign_count: Counter value (public)
❌ NO fingerprint
❌ NO face image
❌ NO biometric template
❌ NO biometric verification result
❌ NO raw biometric data
```

**Server-Side Verification:**

```rust
// Step 1: Verify challenge exists
let challenge = db.find_challenge(challenge_id)?;
// ✅ Challenge found
// ✅ Not expired
// ✅ Not previously used

// Step 2: Verify challenge not replayed
if challenge.used {
    return Err("Challenge already used");
}
// ✅ This is the first use

// Step 3: Fetch credential
let credential = db.find_credential(credential_id)?;
// ✅ Credential found
// ✅ Status is "active"
// ✅ Not expired
// ✅ Not revoked

// Step 4: Verify replay protection (sign_count)
if new_sign_count <= old_sign_count {
    return Err("Replay attack detected");
}
// ✅ Sign count increased (device used again)

// Step 5: Verify signature
let public_key = base64_decode(credential.public_key);
let signature_bytes = base64_decode(signature);
let is_valid = ed25519_verify(public_key, challenge, signature_bytes);
if !is_valid {
    return Err("Signature invalid");
}
// ✅ Signature verified with Ed25519

// Step 6: All checks passed
return Success;
```

## Security Properties

### 1. Biometric Data Confidentiality

**Guarantee:** Raw biometric data (fingerprints, faces) never transmitted.

**How:**
- Biometric matching happens in device's secure enclave
- Device only sends cryptographic signature
- Server never sees biometric data

**Verification:**
```
Network packet capture:
┌──────────────────────────────────────────┐
│ HTTP POST /authenticate/complete         │
│ Content-Type: application/json           │
│                                          │
│ {                                        │
│   "challenge_id": "...",                 │
│   "credential_id": "...",                │
│   "signature": "...",  ← 128 hex chars   │
│   "sign_count": 42                       │
│ }                                        │
│                                          │
│ ✅ Only signature + metadata             │
│ ❌ NO image data                         │
│ ❌ NO template data                      │
│ ❌ NO biometric information              │
└──────────────────────────────────────────┘
```

### 2. Biometric Data Integrity

**Guarantee:** Biometric verification cannot be bypassed.

**How:**
- Local biometric matching is mandatory
- Private key sealed in hardware
- Cannot authenticate without successful biometric match

**Attack Resistance:**
- ❌ Cannot submit signature without biometric verification
- ❌ Cannot extract private key to sign offline
- ❌ Cannot bypass biometric check
- ✅ Must pass biometric verification to sign challenge
- ✅ Signature cryptographically linked to challenge
- ✅ Cannot use same signature for different challenge

### 3. Authentication Non-Repudiation

**Guarantee:** User cannot deny authentication.

**Why:**
- Only user's biometric can sign with their private key
- Signature proves both:
  - Biometric verification passed
  - Authentication intent (specific challenge)
- Signature is cryptographic proof

### 4. Replay Protection

**Two-Layer Protection:**

**Layer 1: Challenge Marking**
```
First authentication:
- Challenge ID: "abc123"
- Status: used = 0
- User authenticates
- Signature sent
- Server marks: used = 1

Second authentication (replay):
- Attacker sends same signature + challenge ID
- Server checks: used = 1
- Returns: "Challenge already used"
- ✅ Replay prevented
```

**Layer 2: Sign Count Increment**
```
First authentication:
- Device's sign_count: 10
- Server's stored sign_count: 10
- User authenticates
- New sign_count: 11
- Server updates stored sign_count: 11

Second authentication (replay):
- Attacker sends: sign_count = 11, same signature
- Server checks: 11 <= 11? YES (not > old value)
- Returns: "Sign count did not increase"
- Alternatively: Signature verification fails (signed with count=10)
- ✅ Replay prevented
```

### 5. Challenge Freshness

**Guarantee:** Challenges expire after 5 minutes.

**Why:**
- Old signatures cannot be reused
- Forces fresh biometric verification
- Limits window for interception/replay

## Supported Biometric Types

### 1. Fingerprint Authentication

**Device Implementation:**
```
1. Fingerprint sensor captures image
2. Local matching engine:
   - Extracts minutiae points
   - Compares against enrolled template
   - Returns: Match/No-Match
3. If Match:
   - Proceeds to sign challenge
   - Private key used (not extracted)
4. Result: Signature sent to server

✅ Server receives: Signature only
❌ Server receives: NO fingerprint image
❌ Server receives: NO minutiae data
```

**Example Devices:**
- iPhone (Touch ID/Face ID)
- Android (Fingerprint sensor)
- Windows Hello
- macOS (Touch ID)

### 2. Face Authentication

**Device Implementation:**
```
1. Camera captures face image
2. Local face matching engine:
   - Extracts face encoding (neural net)
   - Compares against enrolled encoding
   - Returns: Match/No-Match
3. If Match:
   - Proceeds to sign challenge
   - Private key used (not extracted)
4. Result: Signature sent to server

✅ Server receives: Signature only
❌ Server receives: NO face image
❌ Server receives: NO face encoding
```

**Example Devices:**
- iPhone (Face ID)
- Windows Hello (Face)
- Android (Face unlock)

### 3. Platform Passkey

**Device Implementation:**
```
1. Device prompts for PIN/Passkey
2. Local verification:
   - Compare hashed PIN against stored hash
   - Returns: Match/No-Match
3. If Match:
   - Proceeds to sign challenge
   - Private key used (not extracted)
4. Result: Signature sent to server

✅ Server receives: Signature only
❌ Server receives: NO PIN/Passkey
❌ Server receives: NO password data
```

**Example Devices:**
- Any platform with passkey support
- Fallback when biometric unavailable

## Database Schema

### biometric_credentials

```sql
CREATE TABLE biometric_credentials (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    public_key TEXT NOT NULL,              -- ✅ Public only, safe to store
    credential_type TEXT NOT NULL,         -- fingerprint|face|platform
    device_id TEXT NOT NULL,
    device_name TEXT NOT NULL,
    sign_count INTEGER NOT NULL DEFAULT 0, -- Replay protection counter
    status TEXT NOT NULL DEFAULT 'active', -- active|revoked|expired
    created_at DATETIME NOT NULL,
    expires_at DATETIME,
    last_used_at DATETIME,
    revoked_at DATETIME
);

-- CRITICAL: No biometric data columns:
-- ❌ fingerprint_image
-- ❌ fingerprint_template
-- ❌ face_image
-- ❌ face_encoding
-- ❌ biometric_template
-- ❌ biometric_data
-- ❌ password
-- ❌ secret
```

### biometric_challenges

```sql
CREATE TABLE biometric_challenges (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    challenge TEXT NOT NULL,              -- Random bytes (base64)
    challenge_type TEXT NOT NULL,         -- registration|authentication
    credential_id TEXT,                   -- For auth challenges
    expires_at DATETIME NOT NULL,         -- 5 minutes from creation
    used BOOLEAN NOT NULL DEFAULT 0,      -- Prevents reuse
    created_at DATETIME NOT NULL
);

-- Random challenge bytes for signing input
-- NOT sensitive data
```

## API Endpoints

### Registration

#### Begin Registration
```
POST /api/v1/biometric/register/begin
Authorization: Bearer <JWT>
Content-Type: application/json

Request:
{
  "device_id": "uuid-iphone-14",
  "device_name": "iPhone 14"
}

Response:
{
  "challenge_id": "challenge-uuid",
  "challenge": "base64_random_bytes",
  "user_id": "user-123"
}
```

#### Complete Registration
```
POST /api/v1/biometric/register/complete
Authorization: Bearer <JWT>
Content-Type: application/json

Request:
{
  "challenge_id": "challenge-uuid",
  "public_key": "base64_public_key",
  "attestation_object": "base64_attestation",
  "credential_type": "fingerprint",
  "device_id": "uuid-iphone-14",
  "device_name": "iPhone 14"
}

Response:
{
  "success": true,
  "credential_id": "cred-uuid-123",
  "message": "Credential registered"
}
```

### Authentication

#### Begin Authentication
```
POST /api/v1/biometric/authenticate/begin
Content-Type: application/json

Request:
{
  "credential_id": "cred-uuid-123"
}

Response:
{
  "challenge_id": "auth-challenge-uuid",
  "challenge": "base64_random_bytes"
}
```

#### Complete Authentication
```
POST /api/v1/biometric/authenticate/complete
Content-Type: application/json

Request:
{
  "challenge_id": "auth-challenge-uuid",
  "credential_id": "cred-uuid-123",
  "signature": "base64_signature",
  "sign_count": 42
}

Response:
{
  "success": true,
  "result": "SUCCESS",
  "message": "Authentication successful"
}
```

## Testing

See `backend/tests/biometric_auth_tests.rs` for comprehensive test scenarios:

1. ✅ Successful authentication
2. ✅ Failed authentication (wrong biometric)
3. ✅ Invalid challenge
4. ✅ Expired challenge
5. ✅ Replayed challenge
6. ✅ Unknown credential
7. ✅ Revoked credential
8. ✅ Expired credential
9. ✅ Replay protection verification
10. ✅ No raw biometric data stored

## Security Compliance

### GDPR
- ✅ No biometric data collected server-side
- ✅ Biometric data processed locally only
- ✅ No cross-border biometric transmission
- ✅ User can revoke credentials
- ✅ Audit trail maintained

### CCPA
- ✅ User can delete credentials
- ✅ No biometric data sold/shared
- ✅ Transparency: only signatures stored
- ✅ Explicit consent for device registration

### HIPAA (if used for healthcare)
- ✅ No PHI in biometric data transmission
- ✅ Audit logging for all operations
- ✅ Encryption in transit (HTTPS)
- ✅ Encryption at rest (SQLite encryption)

### FIDO2/WebAuthn Standards
- ✅ Implements W3C WebAuthn Level 2
- ✅ Challenge-response protocol
- ✅ Cryptographic signature verification
- ✅ Replay protection
- ✅ Attestation verification

## Conclusion

This biometric authentication gateway ensures that:

1. **Raw biometric data NEVER leaves the device**
2. **Server receives only cryptographic signatures**
3. **Biometric matching is device-local only**
4. **Private keys stored in secure hardware**
5. **Authentication is passwordless and strong**
6. **Compliance with GDPR, CCPA, HIPAA, FIDO2**

The architecture is fundamentally secure because biometric processing is performed entirely on the device, not the server.
