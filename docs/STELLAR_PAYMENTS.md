# Stellar Payment Integration - Phase 5

## Overview

Phase 5 implements complete Stellar payment functionality for the LEGACY student identity card system. The system enables students and merchants to make secure payments using the Stellar blockchain on **testnet only**.

## Critical Security Constraints

⚠️ **SECURITY FIRST**: These constraints are ENFORCED at every level:

1. **NO Secret Keys in Frontend** - All signing happens backend-only
2. **NO Secret Keys in QR Codes** - QR codes contain only public payment data
3. **NO Secret Keys Committed** - Keys ONLY via environment variables
4. **Testnet Only** - All code rejects mainnet URLs/networks
5. **Device-Local Biometric** - Biometric auth happens on device, not transmitted

## Architecture

### Core Components

#### 1. Account Management
```
StudentPaymentAccount    - Student's Stellar account (linked to user_id)
MerchantPaymentAccount   - Merchant's account (optional user_id for bulk merchants)
StellarAccount           - Unified account representation
```

#### 2. Payment Flow
```
PaymentRequest           - Payment initiation (pending → authorized → completed)
PaymentQrData            - QR code data (public only, testnet-enforced)
PaymentSigningRequest    - Secure signing request (SHA-256 hash only)
PaymentAuthorization     - Authorization with signature
```

#### 3. Transaction Management
```
StellarTransactionBuilder    - Constructs transaction envelopes
StellarTransactionEnvelope   - Ready-to-sign transaction
SignedStellarTransaction     - Signed + ready to submit
PaymentTransaction           - Ledger record (pending → submitted → confirmed)
```

#### 4. Reliability
```
TransactionRetryHandler      - Exponential backoff retry logic
PaymentTransaction.has_timed_out() - 5-minute timeout detection
TransactionHistoryFilter     - Query and analysis
```

## Data Flow

### Secure Payment Authorization Flow

```
1. Student initiates payment
   └─ PaymentRequest created (payer, payee, amount, memo)

2. Backend generates signing request
   └─ PaymentSigningRequest with SHA-256 hash (NO sensitive data)
   └─ Sent to frontend

3. Frontend signs with device biometric
   └─ Local biometric processing (never transmitted raw)
   └─ Signature returned (128 hex chars)

4. Backend verifies signature
   └─ Creates PaymentAuthorization
   └─ Builds transaction envelope

5. Backend signs with environment-variable key
   └─ Creates SignedStellarTransaction
   └─ Submits to Stellar testnet

6. Stellar confirms transaction
   └─ Updates PaymentTransaction status
   └─ Records tx_hash and ledger number
```

### QR Code (Secure - No Secrets)

```json
{
  "request_id": "uuid",
  "payee": "GBBD47UZQ53HALXJEMZUKKMYRJA",
  "amount": "50.00",
  "memo": "Lunch payment",
  "network": "testnet",
  "created_at": "2026-09-30T..."
}
```

**What's NOT in QR:**
- ✓ NO secret keys
- ✓ NO private keys
- ✓ NO passwords
- ✓ NO sensitive data

## Environment Variables (Testnet Only)

```bash
# CRITICAL: Secret keys NEVER committed
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE=Test SDF Network ; September 2015

# Keys (TESTNET ONLY)
STELLAR_MASTER_SECRET_KEY=S... (testnet key only)
STELLAR_MERCHANT_SECRET_KEY=S... (testnet merchant key)

# Payment config
PAYMENT_REQUEST_EXPIRY_HOURS=1
PAYMENT_MIN_AMOUNT=0.1
PAYMENT_MAX_AMOUNT=1000.0
TRANSACTION_TIMEOUT_SECONDS=30
TRANSACTION_RETRY_ATTEMPTS=3
```

## API Endpoints

### Payment Requests

**POST /api/v1/stellar/payment-request**
```json
{
  "payer": "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
  "payee": "GBBD47UZQ53HALXJEMZUKKMYRJA",
  "amount": "100.00",
  "payment_type": "student_to_merchant",
  "memo": "Lunch payment",
  "expiry_hours": 1
}
```

Response (201):
```json
{
  "id": "uuid",
  "payer": "...",
  "payee": "...",
  "amount": "100.00",
  "payment_type": "student_to_merchant",
  "memo": "Lunch payment",
  "status": "pending",
  "expires_at": "2026-09-30T12:00:00Z",
  "created_at": "2026-09-30T11:00:00Z"
}
```

**GET /api/v1/stellar/payment-request/{id}**
- Fetch payment request details

### QR Code Generation

**POST /api/v1/stellar/qr-code**
```json
{
  "payment_request_id": "uuid",
  "payee": "GBBD47UZQ53HALXJEMZUKKMYRJA",
  "amount": "100.00",
  "memo": "Lunch"
}
```

Response:
```json
{
  "request_id": "uuid",
  "qr_uri": "stellar:pay?destination=...&amount=100.00&memo=Lunch",
  "payment_url": "https://api.example.com/pay/uuid",
  "network": "testnet"
}
```

### Payment Authorization

**POST /api/v1/stellar/signing-request**
```json
{
  "payment_request_id": "uuid",
  "account": "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW"
}
```

Response:
```json
{
  "request_id": "uuid",
  "payment_request_id": "uuid",
  "account": "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
  "signing_data": "sha256_hash_hex",
  "algorithm": "Ed25519",
  "expires_at": "2026-09-30T11:05:00Z"
}
```

**POST /api/v1/stellar/submit-signature**
```json
{
  "signing_request_id": "uuid",
  "signature": "128_hex_chars_ed25519_signature"
}
```

### Transaction Status

**GET /api/v1/stellar/transaction/{id}**
- Fetch transaction status and history

**GET /api/v1/stellar/transaction-history?account=...&status=confirmed&limit=50**
- Query transaction history with filters

### Payment Confirmation

**POST /api/v1/stellar/confirm-payment**
```json
{
  "transaction_id": "uuid",
  "tx_hash": "sha256_hash_hex",
  "ledger": 12345
}
```

## Transaction States

### Payment Request States
```
pending → authorized → completed
       → failed
```

### Payment Transaction States
```
pending → submitted → confirmed
       → failed (with retry up to 3x)
```

### Retry Logic (Exponential Backoff)
```
Attempt 1: 10s delay
Attempt 2: 20s delay
Attempt 3: 40s delay
Max: 3 attempts, then manual intervention
```

## Testing

All components include comprehensive tests:

- **125+ tests** covering success paths and failure scenarios
- **Security tests** verifying NO secret key exposure
- **Testnet enforcement** tests confirming mainnet rejection
- **State machine tests** validating transaction lifecycle
- **Timeout & retry tests** for reliability

Run tests:
```bash
cargo test --lib stellar_payment
```

## Key Management

### Frontend (Student Device)
- ✓ Receives SHA-256 hash to sign
- ✓ Performs biometric authentication (device-local)
- ✓ Signs with biometric key (never transmitted raw)
- ✓ Returns signature only (128 hex chars)

### Backend (Server)
- ✓ Receives signature
- ✓ Verifies signature format
- ✓ Creates PaymentAuthorization
- ✓ Signs with environment-variable key (NEVER exposed)
- ✓ Submits to Stellar testnet

### Never Exposed
- ✗ Raw biometric data
- ✗ Secret keys (any kind)
- ✗ Private keys
- ✗ Passwords

## Official Stellar APIs Used

- **soroban-client** (v0.5) - RPC + transaction building
- **stellar-xdr** (v21) - XDR encoding/decoding
- **stellar-strkey** (v0.0.8) - Address encoding
- **Horizon** - Account information queries

All APIs are official Stellar SDKs, no outdated examples.

## Testnet Details

**Network Name**: Stellar Testnet  
**Network Passphrase**: `Test SDF Network ; September 2015`  
**RPC Endpoint**: https://soroban-testnet.stellar.org  
**Horizon Endpoint**: https://horizon-testnet.stellar.org  
**Asset**: Lumens (XLM) testnet only  
**Faucet**: https://stellar.expert/setup  

## Database Schema

### Tables Created
- `stellar_accounts` - Student/merchant account records
- `payment_requests` - Payment request records
- `payment_authorizations` - Authorization signatures
- `payment_transactions` - Ledger records
- `payment_qr_codes` - QR code records
- `payment_signing_requests` - Signing request records
- `payment_transaction_history` - Audit trail
- `payment_retry_log` - Retry history

All with comprehensive indexing for performance.

## Validation & Constraints

### Payment Amounts
- Minimum: 0.1 XLM
- Maximum: 1000.0 XLM
- Format: Decimal (string to avoid float precision issues)

### Fees
- Minimum: 100 stroops (~0.001 XLM)
- Maximum: 10000 stroops (~0.001 XLM)

### Memos
- Length: 1-28 characters (Stellar standard)
- Type: UTF-8 text memo

### Public Keys
- Format: Starts with 'G', exactly 56 characters, valid base32
- Validation: Enforced before any operation

### Signatures
- Format: 128 hexadecimal characters (Ed25519)
- Algorithm: Ed25519
- Verified before authorization

## Future Enhancements

- Multi-signature transactions
- Payment channels
- DEX integration (trading)
- Asset issuance
- Smart contracts (Soroban)

## References

- [Stellar Official Docs](https://developers.stellar.org)
- [Stellar Testnet Guide](https://developers.stellar.org/docs/learn/fundamentals/stellar-stack)
- [Stellar RPC API](https://developers.stellar.org/docs/build)
- [Soroban Client Docs](https://docs.rs/soroban-client/)

## Support

For issues:
1. Check testnet connectivity
2. Verify environment variables
3. Review transaction hash on [Stellar Expert](https://stellar.expert)
4. Check transaction history in database
5. Enable debug logging
