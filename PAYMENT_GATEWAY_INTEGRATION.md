# Payment Gateway Integration Guide

**Quick Reference**: How to integrate biometric authentication with Stellar payments

---

## Core Files

### 1. Payment Gateway Module
**File**: `backend/src/payment_gateway.rs` (600+ lines)

**Components**:
- `PaymentAuthenticationGateway` - Main gateway with 8 check functions
- `PaymentGatewayAuthorization` - Authorization result with audit trail
- `GatewayCheckResult` - Individual check result
- `AntiReplayTracker` - Prevents duplicate payments
- `PaymentAuthenticationRequest` - Request structure

**Key Functions**:
```rust
// Run all 8 security checks
pub async fn authenticate_payment(
    req: &PaymentAuthenticationRequest,
    credential: &Option<Credential>,
    merchant_verified: bool,
    challenge: &Option<AuthenticationChallenge>,
    replay_tracker: &AntiReplayTracker,
    daily_limit: f64,
    daily_used: f64,
) -> PaymentGatewayAuthorization
```

### 2. REST Handlers
**File**: `backend/src/handlers/payment_auth.rs` (300+ lines)

**Endpoints**:
```
POST   /api/v1/payment/authenticate     (Run gateway checks)
POST   /api/v1/payment/submit           (Submit to Stellar after auth)
GET    /api/v1/stellar/authorization/{id} (Check auth status)
```

### 3. Database Schema
**File**: `backend/migrations/20240103000001_payment_gateway.sql`

**Tables**:
- `payment_gateway_authorizations` - Authorization records
- `payment_gateway_checks` - Individual check results (audit trail)
- `payment_anti_replay_log` - Prevent duplicate payments
- `payment_policies` - Student daily limits
- `payment_gateway_audit_log` - Complete payment attempt log

### 4. Test Suite
**File**: `backend/tests/payment_gateway_tests.rs`

**26 tests** proving every failure condition blocks payment:
```
✓ Biometric failure → NO payment
✓ Credential missing → NO payment
✓ Credential revoked → NO payment
✓ Credential expired → NO payment
✓ Credential suspended → NO payment
✓ Challenge expired → NO payment
✓ Challenge reused → NO payment
✓ Duplicate payment → NO payment
✓ Merchant not verified → NO payment
✓ Amount too low → NO payment
✓ Amount too high → NO payment
✓ Daily limit exceeded → NO payment
... and 14 more
```

---

## Integration Steps

### Step 1: Run Database Migration
```bash
cd backend
sqlx migrate run
# Creates 8 new tables with 20+ indexes
```

### Step 2: Start Server
```bash
cargo run --release
```

### Step 3: Call Payment Gateway

**Complete Flow**:

```bash
# 1. Authenticate Payment (8 security checks)
curl -X POST http://localhost:8080/api/v1/payment/authenticate \
  -H "Content-Type: application/json" \
  -d '{
    "payment_request_id": "payment_123",
    "student_credential_id": "cred_456",
    "student_user_id": "user_789",
    "merchant_account": "GMERCHANT_56chars_here",
    "payment_amount": "50.0",
    "biometric_challenge_id": "challenge_111",
    "biometric_signature": "abc123def456...",
    "biometric_credential_id": "biocred_222"
  }'

# Response if all checks pass:
{
  "authorized": true,
  "authorization_id": "auth_abc123",
  "checks": [
    {"check_name": "Credential Verification", "passed": true},
    {"check_name": "Credential Status Check", "passed": true},
    ...8 total checks...
  ],
  "authorized_at": "2026-09-30T14:32:00Z",
  "expires_at": "2026-09-30T14:47:00Z"
}

# 2. Submit Payment (ONLY after authorization)
curl -X POST http://localhost:8080/api/v1/payment/submit \
  -H "Content-Type: application/json" \
  -d '{
    "authorization_id": "auth_abc123",
    "payment_request_id": "payment_123",
    "transaction_envelope": "AAAAAgAAAAB..."
  }'

# Response:
{
  "tx_hash": "abc123def456...",
  "ledger": 12345,
  "status": "submitted",
  "submitted_at": "2026-09-30T14:32:15Z"
}
```

---

## Security Checks (In Order)

| # | Check | Fails If | Response |
|---|-------|----------|----------|
| 1 | Credential Verification | Not found | 403 |
| 2 | Credential Status | Revoked/Expired/Suspended/Pending | 403 |
| 3 | Merchant Authorization | Not verified | 403 |
| 4 | Payment Amount | < 0.1 or > 1000 XLM | 403 |
| 5 | Biometric/Passkey | Invalid signature | 403 |
| 6 | Challenge Verification | Expired or reused | 403 |
| 7 | Anti-Replay Check | Duplicate payment_request_id | 403 |
| 8 | Payment Policy | Daily limit exceeded | 403 |

---

## Failure Scenarios

### Scenario 1: Biometric Authentication Fails
```bash
# Request with invalid biometric signature (wrong length)
{
  "biometric_signature": "invalid_too_short"
}

# Response
{
  "authorized": false,
  "failure_summary": "Biometric/Passkey Authentication: Invalid biometric signature format",
  "checks": [
    {"check_name": "Biometric/Passkey Authentication", "passed": false}
  ]
}

# Stellar: NOT called ✓
# Transaction: NOT submitted ✓
```

### Scenario 2: Credential Revoked
```bash
# Database shows credential status = "revoked"

# Response
{
  "authorized": false,
  "failure_summary": "Credential Status Check: Credential has been revoked",
  "checks": [
    {"check_name": "Credential Status Check", "passed": false}
  ]
}

# Stellar: NOT called ✓
# Transaction: NOT submitted ✓
```

### Scenario 3: Duplicate Payment (Anti-Replay)
```bash
# First payment with payment_request_id = "payment_123" → SUCCESS

# Second payment with same payment_request_id = "payment_123"
{
  "payment_request_id": "payment_123"
}

# Response
{
  "authorized": false,
  "failure_summary": "Anti-Replay Check: Payment request has already been processed",
  "checks": [
    {"check_name": "Anti-Replay Check", "passed": false}
  ]
}

# Stellar: NOT called ✓
# Transaction: NOT submitted ✓
```

### Scenario 4: Challenge Expired
```bash
# Challenge created 10 minutes ago (expires after 5 minutes)

# Response
{
  "authorized": false,
  "failure_summary": "Challenge Verification: Challenge has expired",
  "checks": [
    {"check_name": "Challenge Verification", "passed": false}
  ]
}

# Stellar: NOT called ✓
# Transaction: NOT submitted ✓
```

### Scenario 5: Daily Limit Exceeded
```bash
# Student already used 480 XLM today, daily limit 500 XLM
# Trying to pay 50 XLM (would use 530 total)

# Response
{
  "authorized": false,
  "failure_summary": "Payment Policy Check: Daily limit exceeded",
  "checks": [
    {"check_name": "Payment Policy Check", "passed": false}
  ]
}

# Stellar: NOT called ✓
# Transaction: NOT submitted ✓
```

---

## Success Scenario

```bash
# All checks pass
POST /api/v1/payment/authenticate
{
  "payment_request_id": "payment_xyz",
  "student_credential_id": "cred_123",
  "student_user_id": "user_456",
  "merchant_account": "GMERCHANT...",
  "payment_amount": "50.0",
  "biometric_challenge_id": "challenge_789",
  "biometric_signature": "a" repeated 128 times,  // valid Ed25519
  "biometric_credential_id": "biocred_111"
}

# Response: 200 OK
{
  "authorized": true,
  "authorization_id": "auth_success_123",
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

# Now can submit to Stellar:
POST /api/v1/payment/submit
{
  "authorization_id": "auth_success_123",
  "payment_request_id": "payment_xyz",
  "transaction_envelope": "AAAAAgAAAAB..."
}

# Response: 202 Accepted
{
  "tx_hash": "abc123...",
  "ledger": 12345,
  "status": "submitted",
  "submitted_at": "2026-09-30T14:32:15Z"
}

# Stellar: CALLED ✓
# Transaction: SUBMITTED ✓
```

---

## Fail-Closed Architecture

```
Payment Request
  ↓
Payment Authentication Gateway
  ├─ Check 1: Credential exists? → YES continue
  ├─ Check 2: Credential active? → YES continue
  ├─ Check 3: Merchant verified? → YES continue
  ├─ Check 4: Amount valid? → YES continue
  ├─ Check 5: Biometric valid? → YES continue
  ├─ Check 6: Challenge valid? → YES continue
  ├─ Check 7: Anti-replay pass? → YES continue
  └─ Check 8: Policy allows? → YES continue
     ↓
  ALL CHECKS PASSED ✓
     ↓
  Authorization Created (15 min window)
     ↓
  Submit to Stellar ✓
     ↓
  Transaction Confirmed

---

Alternative:

Payment Request
  ↓
Payment Authentication Gateway
  ├─ Check 1: Credential exists? → NO
     ↓
  STOP - AUTHORIZATION DENIED
     ↓
  Stellar: NOT called ✓
  Transaction: NOT submitted ✓
  Response: 403 Forbidden
```

---

## Code Example

### Frontend: Request Payment Authentication

```typescript
const response = await fetch('/api/v1/payment/authenticate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    payment_request_id: 'payment_' + Date.now(),
    student_credential_id: student.credentialId,
    student_user_id: student.userId,
    merchant_account: merchant.stellarAddress,
    payment_amount: '50.0',
    biometric_challenge_id: challenge.id,
    biometric_signature: signature,  // from biometric device
    biometric_credential_id: bioCredId
  })
});

if (!response.ok) {
  // Check failed - show error
  const auth = await response.json();
  console.error('Payment denied:', auth.failure_summary);
  return;
}

const auth = await response.json();
console.log('Payment authorized:', auth.authorization_id);

// Now submit to Stellar
const stellarResponse = await fetch('/api/v1/payment/submit', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    authorization_id: auth.authorization_id,
    payment_request_id: auth.payment_request_id,
    transaction_envelope: txEnvelope
  })
});
```

---

## Testing

### Run All Gateway Tests
```bash
cargo test payment_gateway_tests
```

### Run Specific Test
```bash
cargo test test_fail_if_biometric_authentication_fails
cargo test test_fail_if_credential_revoked
cargo test test_fail_if_challenge_reused_replay_attack
```

### Output
```
test test_fail_if_biometric_authentication_fails ... ok ✓ Payment REJECTED: Biometric authentication failed
test test_fail_if_credential_missing ... ok ✓ Payment REJECTED: Credential missing
test test_fail_if_credential_revoked ... ok ✓ Payment REJECTED: Credential revoked
... 23 more tests passing ...

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Monitoring & Audit

### Authorization Metrics
```sql
-- Count authorized vs denied payments
SELECT 
  authorized,
  COUNT(*) as count
FROM payment_gateway_authorizations
GROUP BY authorized;

-- Find most common failure reasons
SELECT 
  check_name,
  COUNT(*) as failures
FROM payment_gateway_checks
WHERE passed = false
GROUP BY check_name
ORDER BY failures DESC;

-- Payment attempt timeline
SELECT 
  payment_request_id,
  authorization_result,
  failure_summary,
  attempted_at
FROM payment_gateway_audit_log
ORDER BY attempted_at DESC
LIMIT 10;
```

---

## Deployment Checklist

- [ ] Payment gateway module compiles
- [ ] All 26 tests pass
- [ ] Database migrations applied
- [ ] Anti-replay tracker initialized
- [ ] Stellar testnet configured
- [ ] Student daily limits populated
- [ ] Audit logging verified
- [ ] Error responses tested
- [ ] Authorization expiration tested
- [ ] Challenge expiration tested
- [ ] Duplicate payment prevention tested

---

## Critical Security Guarantees

1. **NO Stellar RPC call without gateway authorization** ✓
2. **NO payment if credential revoked** ✓
3. **NO payment if credential expired** ✓
4. **NO payment if credential suspended** ✓
5. **NO payment if biometric fails** ✓
6. **NO payment if challenge expired** ✓
7. **NO payment if challenge reused** ✓
8. **NO duplicate payment (same payment_request_id)** ✓
9. **NO payment exceeding daily limit** ✓
10. **NO payment to unverified merchant** ✓

---

## Support

For questions about payment gateway integration:
- See `docs/PAYMENT_GATEWAY.md` for comprehensive reference
- See `backend/src/payment_gateway.rs` for implementation details
- Run `cargo test payment_gateway_tests` to verify all checks
- Review `backend/tests/payment_gateway_tests.rs` for failure scenarios

