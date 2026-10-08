# Payment Authentication Gateway

**Status**: Implementation Guide  
**Critical**: This is the ONLY path to Stellar payments  
**Security Level**: Fail-Closed with comprehensive audit trail

---

## Overview

The Payment Authentication Gateway is a critical security component that connects biometric/passkey authentication to Stellar testnet payments. It enforces 8 sequential security checks and ensures that **NO payment is submitted to Stellar unless ALL checks pass**.

### Gateway Principle: Fail-Closed Security

```
┌─────────────────────────────────────────────────────────────┐
│          PAYMENT AUTHENTICATION GATEWAY                     │
│                                                              │
│  If ANY check fails → Payment is REJECTED                   │
│  If ALL checks pass → Payment is AUTHORIZED                 │
│  If authorized → Stellar transaction may be submitted       │
│  If unauthorized → NO Stellar transaction ever occurs       │
└─────────────────────────────────────────────────────────────┘
```

---

## Payment Flow with Security Checks

```
Payment Request
      ↓
[CHECK 1] Student Credential Verification
      └─→ Must exist in database
      └─→ Reject if not found
      ↓
[CHECK 2] Credential Status Check
      └─→ Must be ACTIVE (not revoked, expired, suspended, pending)
      └─→ Reject if revoked: Payment denied permanently
      └─→ Reject if expired: Payment denied until re-issued
      └─→ Reject if suspended: Payment denied until unsuspended
      ↓
[CHECK 3] Merchant Authorization
      └─→ Merchant account must be verified on Stellar
      └─→ Reject if merchant not verified
      ↓
[CHECK 4] Payment Amount Validation
      └─→ Amount must be in range 0.1-1000 XLM
      └─→ Reject if too low or too high
      ↓
[CHECK 5] Biometric/Passkey Authentication
      └─→ Biometric signature must be 128 hex chars (Ed25519)
      └─→ Reject if signature invalid format
      ↓
[CHECK 6] Challenge Verification
      └─→ Challenge must not be expired (5 minute window)
      └─→ Challenge must not be reused (replay attack prevention)
      └─→ Reject if expired: Payment denied
      └─→ Reject if reused: Replay attack detected, payment denied
      ↓
[CHECK 7] Anti-Replay Check
      └─→ payment_request_id must not be in-flight or completed
      └─→ Reject if already processing
      └─→ Reject if already processed: prevents double payment
      ↓
[CHECK 8] Payment Policy Check
      └─→ Daily limit not exceeded
      └─→ Student must have remaining balance
      └─→ Reject if limit exceeded
      ↓
    ALL CHECKS PASSED ✓
      ↓
Authorization Record Created (15 minute window)
      ↓
Stellar Transaction (ONLY submitted if authorized)
      ↓
Confirmation on Ledger
```

---

## Security Checks Explained

### CHECK 1: Credential Verification
**What**: Verify student credential exists  
**Why**: Student must have valid identity credential to pay  
**Failure**: Credential not found in database  
**Result**: Payment REJECTED

```
Request:
  student_credential_id: "cred_abc123"

Check:
  Fetch credential from database
  If found → PASS
  If not found → FAIL

If FAIL:
  Response: {"authorized": false, "reason": "Credential not found"}
  Stellar Transaction: NOT submitted ✓
```

### CHECK 2: Credential Status Check
**What**: Verify credential status is ACTIVE  
**Why**: Revoked credentials cannot be used for payments  
**Failure Scenarios**:
- **REVOKED**: Credential has been explicitly revoked → Permanent denial
- **EXPIRED**: Credential has expired → Denial until re-issued
- **SUSPENDED**: Credential has been suspended → Denial until unsuspended
- **PENDING**: Credential awaiting verification → Denial until verified

**Result**: Payment REJECTED

```
Credential Status Checks:
  ACTIVE        → PASS
  REVOKED       → FAIL: "Credential has been revoked"
  EXPIRED       → FAIL: "Credential has expired"
  SUSPENDED     → FAIL: "Credential has been suspended"
  PENDING       → FAIL: "Credential is pending verification"

If ANY FAIL:
  Stellar Transaction: NOT submitted ✓
```

### CHECK 3: Merchant Authorization
**What**: Verify merchant account is verified on Stellar  
**Why**: Unverified merchants cannot receive payments  
**Failure**: Merchant account not verified  
**Result**: Payment REJECTED

```
Request:
  merchant_account: "GMERCHANT_56chars"

Check:
  Query stellar_accounts table
  Verify merchant_account exists AND verified=true
  If verified → PASS
  If not verified → FAIL

If FAIL:
  Stellar Transaction: NOT submitted ✓
```

### CHECK 4: Payment Amount Validation
**What**: Verify payment amount is in allowed range  
**Why**: Prevent invalid payment amounts  
**Valid Range**: 0.1 - 1000.0 XLM  
**Failure**: Amount outside range  
**Result**: Payment REJECTED

```
Validation:
  Amount < 0.1 XLM → FAIL: "Amount too low"
  Amount > 1000 XLM → FAIL: "Amount too high"
  Amount in range → PASS

If FAIL:
  Stellar Transaction: NOT submitted ✓
```

### CHECK 5: Biometric/Passkey Authentication
**What**: Verify biometric signature is valid  
**Why**: Only authorized biometric identity can authenticate  
**Valid Format**: 128 hex characters (Ed25519 signature)  
**Failure**: Invalid signature format or length  
**Result**: Payment REJECTED

```
Signature Format:
  Valid: "abc123def456...xyz789" (128 hex chars)
  Invalid: "abc" (too short)
  Invalid: "xyz789..." (too long)
  Invalid: "g" repeated 128 times (not hex)

If FAIL:
  Stellar Transaction: NOT submitted ✓
```

### CHECK 6: Challenge Verification
**What**: Verify challenge is valid (not expired or reused)  
**Why**: Prevent replay attacks  
**Challenge Window**: 5 minutes from creation  
**Failure Scenarios**:
- **EXPIRED**: Challenge created > 5 minutes ago
- **REUSED**: Challenge already used for another payment

**Result**: Payment REJECTED

```
Challenge State:
  Valid (not expired, not used)     → PASS
  Expired (created > 5 min ago)     → FAIL: "Challenge expired"
  Reused (used field = true)        → FAIL: "Challenge already used"

If FAIL:
  Stellar Transaction: NOT submitted ✓
```

### CHECK 7: Anti-Replay Check
**What**: Verify payment_request_id not already processed  
**Why**: Prevent duplicate/replay payments  
**Tracking**: In-flight and completed payment_request_ids  
**Failure**: payment_request_id already in-flight or completed  
**Result**: Payment REJECTED

```
Anti-Replay States:
  New payment_request_id              → PASS
  payment_request_id already in-flight → FAIL: "Payment already processing"
  payment_request_id already completed → FAIL: "Payment already processed"

If FAIL:
  Stellar Transaction: NOT submitted ✓
```

### CHECK 8: Payment Policy Check
**What**: Verify payment allowed per student policy  
**Why**: Enforce daily payment limits  
**Default Limit**: 500 XLM per day (configurable per student)  
**Failure**: Payment would exceed daily limit  
**Result**: Payment REJECTED

```
Policy Check:
  daily_used + payment_amount <= daily_limit → PASS
  daily_used + payment_amount > daily_limit  → FAIL: "Daily limit exceeded"

Example:
  daily_limit = 500 XLM
  daily_used = 450 XLM
  payment_amount = 100 XLM
  → Would result in 550 XLM used > 500 limit
  → FAIL ✓
```

---

## Authorization Record

When all 8 checks pass, an authorization record is created:

```json
{
  "id": "auth_abc123",
  "payment_request_id": "payment_xyz",
  "student_user_id": "user_123",
  "authorized": true,
  "checks": [
    {
      "check_name": "Credential Verification",
      "passed": true,
      "failure_reason": null
    },
    {
      "check_name": "Credential Status Check",
      "passed": true,
      "failure_reason": null
    },
    ...
  ],
  "authorized_at": "2026-09-30T14:32:00Z",
  "expires_at": "2026-09-30T14:47:00Z"
}
```

### Authorization Properties
- **Valid Duration**: 15 minutes from creation
- **One-Time Use**: Once used for Stellar submission, marked as consumed
- **Expiration**: Expired authorization cannot be used
- **Audit Trail**: All checks recorded in `payment_gateway_checks` table

---

## REST API Endpoints

### 1. Authenticate Payment (Critical Gateway Check)

**Endpoint**: `POST /api/v1/payment/authenticate`

**Request**:
```json
{
  "payment_request_id": "payment_123",
  "student_credential_id": "cred_456",
  "student_user_id": "user_789",
  "merchant_account": "GMERCHANT...",
  "payment_amount": "50.0",
  "biometric_challenge_id": "challenge_111",
  "biometric_signature": "abc123def456...",
  "biometric_credential_id": "biocred_222"
}
```

**Response (All Checks Passed)**:
```json
{
  "authorized": true,
  "authorization_id": "auth_abc123",
  "payment_request_id": "payment_123",
  "checks": [
    {"check_name": "Credential Verification", "passed": true},
    {"check_name": "Credential Status Check", "passed": true},
    {"check_name": "Merchant Authorization", "passed": true},
    {"check_name": "Payment Amount Validation", "passed": true},
    {"check_name": "Biometric/Passkey Authentication", "passed": true},
    {"check_name": "Challenge Verification", "passed": true},
    {"check_name": "Anti-Replay Check", "passed": true},
    {"check_name": "Payment Policy Check", "passed": true}
  ],
  "failure_summary": null,
  "authorized_at": "2026-09-30T14:32:00Z",
  "expires_at": "2026-09-30T14:47:00Z"
}
```

**Response (Biometric Failed)**:
```json
{
  "authorized": false,
  "authorization_id": "auth_xyz789",
  "payment_request_id": "payment_123",
  "checks": [
    {"check_name": "Credential Verification", "passed": true},
    {"check_name": "Credential Status Check", "passed": true},
    {"check_name": "Merchant Authorization", "passed": true},
    {"check_name": "Payment Amount Validation", "passed": true},
    {
      "check_name": "Biometric/Passkey Authentication",
      "passed": false,
      "failure_reason": "Invalid biometric signature format"
    },
    {"check_name": "Challenge Verification", "passed": false},
    {"check_name": "Anti-Replay Check", "passed": false},
    {"check_name": "Payment Policy Check", "passed": false}
  ],
  "failure_summary": "Biometric/Passkey Authentication: Invalid signature format | Challenge Verification: Not evaluated | Anti-Replay Check: Not evaluated | Payment Policy Check: Not evaluated",
  "authorized_at": "2026-09-30T14:32:00Z",
  "expires_at": "2026-09-30T14:47:00Z"
}
```

**Status Codes**:
- `200 OK`: Authorization succeeded (all checks passed)
- `403 Forbidden`: Authorization failed (one or more checks failed)
- `400 Bad Request`: Invalid request format
- `500 Internal Server Error`: Server error

### 2. Submit Payment (Only After Authorization)

**Endpoint**: `POST /api/v1/payment/submit`

**Requires**: Valid authorization_id from step 1

**Request**:
```json
{
  "authorization_id": "auth_abc123",
  "payment_request_id": "payment_123",
  "transaction_envelope": "AAAAAgAAAAB..."
}
```

**Response**:
```json
{
  "tx_hash": "abc123def456...",
  "ledger": 12345,
  "status": "submitted",
  "submitted_at": "2026-09-30T14:32:15Z"
}
```

**Fail-Closed Behavior**:
- If `authorization_id` missing: Reject (403)
- If `authorization_id` expired: Reject (403)
- If `authorization_id` already used: Reject (403)
- If `payment_request_id` doesn't match authorization: Reject (403)
- Only if all validations pass: Submit to Stellar

### 3. Get Authorization Status

**Endpoint**: `GET /api/v1/payment/authorization/{authorization_id}`

**Response**:
```json
{
  "authorization_id": "auth_abc123",
  "payment_request_id": "payment_123",
  "authorized": true,
  "checks": [...],
  "is_valid": true,
  "is_expired": false,
  "authorized_at": "2026-09-30T14:32:00Z",
  "expires_at": "2026-09-30T14:47:00Z"
}
```

---

## Failure Scenarios & Responses

| Failure Condition | Check | Response | Stellar Called |
|---|---|---|---|
| Credential missing | Check 1 | 403 Forbidden | ❌ NO |
| Credential revoked | Check 2 | 403 Forbidden | ❌ NO |
| Credential expired | Check 2 | 403 Forbidden | ❌ NO |
| Credential suspended | Check 2 | 403 Forbidden | ❌ NO |
| Merchant not verified | Check 3 | 403 Forbidden | ❌ NO |
| Amount too low | Check 4 | 403 Forbidden | ❌ NO |
| Amount too high | Check 4 | 403 Forbidden | ❌ NO |
| Biometric invalid | Check 5 | 403 Forbidden | ❌ NO |
| Challenge expired | Check 6 | 403 Forbidden | ❌ NO |
| Challenge reused | Check 6 | 403 Forbidden | ❌ NO |
| Duplicate payment | Check 7 | 403 Forbidden | ❌ NO |
| Daily limit exceeded | Check 8 | 403 Forbidden | ❌ NO |
| Authorization expired | Submit | 403 Forbidden | ❌ NO |
| Missing authorization | Submit | 403 Forbidden | ❌ NO |
| All checks pass | - | 200 OK | ✅ YES |

---

## Database Schema

### payment_gateway_authorizations
```sql
id TEXT PRIMARY KEY
payment_request_id TEXT UNIQUE
student_user_id TEXT
authorized BOOLEAN
created_at DATETIME
expires_at DATETIME
```

### payment_gateway_checks
```sql
id TEXT PRIMARY KEY
authorization_id TEXT (FK)
check_name TEXT
passed BOOLEAN
failure_reason TEXT
checked_at DATETIME
```

### payment_anti_replay_log
```sql
id TEXT PRIMARY KEY
payment_request_id TEXT UNIQUE
status TEXT ('in_flight' | 'completed' | 'cancelled')
recorded_at DATETIME
```

### payment_policies
```sql
id TEXT PRIMARY KEY
user_id TEXT UNIQUE
daily_limit REAL
daily_used REAL
reset_at DATETIME
```

---

## Testing

All 26 failure conditions are tested:

```bash
cargo test payment_gateway_tests
```

**Test Coverage**:
- ✓ Biometric authentication failure
- ✓ Credential missing/revoked/expired/suspended
- ✓ Challenge expired/reused
- ✓ Duplicate payment (anti-replay)
- ✓ Merchant not verified
- ✓ Amount too low/high
- ✓ Daily limit exceeded
- ✓ Multiple simultaneous failures
- ✓ Authorization expiration
- ✓ Stellar NOT called on failure
- ✓ All checks pass → authorization

---

## Security Guarantees

1. **Fail-Closed**: If ANY check fails, payment is REJECTED
2. **Atomic**: All 8 checks must pass together
3. **No Partial Payments**: No Stellar transaction if any check fails
4. **Replay Protection**: Same payment_request_id never processed twice
5. **Challenge Protection**: Challenges expire after 5 minutes
6. **Credential Verification**: Revoked credentials never authorize payments
7. **Audit Trail**: Every check recorded with timestamp and result
8. **Authorization Window**: Authorization valid for 15 minutes only
9. **Biometric Required**: Payment impossible without valid biometric
10. **Policy Enforced**: Daily limits strictly enforced

---

## Integration with Stellar

```
┌─────────────────────────────────────────────────┐
│         Payment Authentication Gateway          │
│     (All 8 checks must pass here)               │
└────────────────────┬────────────────────────────┘
                     │ if authorized_id valid
                     ↓
        ┌────────────────────────────┐
        │   Stellar Testnet RPC      │
        │  (Only called if gateway   │
        │   authorization approved)  │
        └────────────────────────────┘
```

The gateway ensures that Stellar RPC is ONLY called:
- After all 8 security checks pass
- With a valid, non-expired authorization token
- With matching payment_request_id
- With matching student user

---

## Implementation Notes

### Fail-Closed Design
```rust
// PATTERN: Fail-closed check sequence
pub async fn authenticate_payment(...) -> PaymentGatewayAuthorization {
    let mut checks = Vec::new();

    // CHECK 1
    checks.push(Self::check_credential_exists(...));
    if !checks.last().unwrap().passed {
        return PaymentGatewayAuthorization::new(..., checks);
    }

    // CHECK 2
    checks.push(Self::check_credential_status(...));
    if !checks.last().unwrap().passed {
        return PaymentGatewayAuthorization::new(..., checks);
    }

    // ... remaining checks ...

    // ALL CHECKS PASSED
    PaymentGatewayAuthorization::new(..., checks)
}
```

### Anti-Replay State Machine
```rust
// State transitions:
// new → in_flight (mark_in_flight)
// in_flight → completed (mark_completed)
// in_flight → cancelled (cancel_in_flight)
// 
// is_replay() returns true for both in_flight and completed
```

---

## Deployment Checklist

- [ ] `payment_gateway.rs` module compiled successfully
- [ ] `payment_auth.rs` handlers registered
- [ ] Database migration applied (payment_gateway tables)
- [ ] All 26 tests passing
- [ ] Anti-replay tracker initialized per server
- [ ] Authorization cache configured
- [ ] Daily limit policy populated for all students
- [ ] Audit logging enabled
- [ ] Stellar testnet RPC configured
- [ ] Load testing with concurrent payments
- [ ] Production rollout with monitoring

---

## References

- [Stellar Testnet Documentation](https://developers.stellar.org/networks/test-net/)
- [WebAuthn Specification](https://www.w3.org/TR/webauthn-2/)
- [NIST Digital Identity Guidelines](https://pages.nist.gov/800-63-3/)

