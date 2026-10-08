# Stellar Payment System - Quick Start Guide

## 5-Minute Setup

### 1. Get Testnet Keys

Go to [Stellar Expert](https://stellar.expert/setup) or use Soroban CLI:

```bash
soroban keys generate student_key --network testnet
soroban keys generate merchant_key --network testnet
```

This gives you testnet keys only (safe for development).

### 2. Set Environment Variables

Create `.env` from `.env.example`:

```bash
cp backend/.env.example backend/.env
```

Update with your testnet keys:

```bash
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_MASTER_SECRET_KEY=S... (your testnet key)
STELLAR_MERCHANT_SECRET_KEY=S... (another testnet key)
```

⚠️ **NEVER commit these keys!**

### 3. Run Migrations

```bash
cd backend
sqlx migrate run
```

This creates all payment tables.

### 4. Start Backend

```bash
cargo run
```

### 5. Test Endpoints

**Health Check**:
```bash
curl http://localhost:8080/api/v1/stellar/health
```

**Create Payment Request**:
```bash
curl -X POST http://localhost:8080/api/v1/stellar/payment-request \
  -H "Content-Type: application/json" \
  -d '{
    "payer": "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
    "payee": "GBBD47UZQ53HALXJEMZUKKMYRJA",
    "amount": "50.00",
    "payment_type": "student_to_merchant",
    "memo": "Lunch",
    "expiry_hours": 1
  }'
```

**Generate QR Code**:
```bash
curl -X POST http://localhost:8080/api/v1/stellar/qr-code \
  -H "Content-Type: application/json" \
  -d '{
    "payment_request_id": "uuid-from-above",
    "payee": "GBBD47UZQ53HALXJEMZUKKMYRJA",
    "amount": "50.00",
    "memo": "Lunch"
  }'
```

---

## Full Payment Flow

### Step 1: Create Payment Request

```json
POST /api/v1/stellar/payment-request

{
  "payer": "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
  "payee": "GBBD47UZQ53HALXJEMZUKKMYRJA",
  "amount": "100.00",
  "payment_type": "student_to_merchant",
  "memo": "Tuition payment"
}
```

Response:
```json
{
  "id": "payment-uuid",
  "status": "pending",
  "expires_at": "2026-09-30T12:00:00Z"
}
```

### Step 2: Generate Authorization Request

```json
POST /api/v1/stellar/signing-request

{
  "payment_request_id": "payment-uuid",
  "account": "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW"
}
```

Response:
```json
{
  "request_id": "signing-uuid",
  "signing_data": "sha256_hash_hex",
  "algorithm": "Ed25519",
  "expires_at": "2026-09-30T11:05:00Z"
}
```

### Step 3: Frontend Signs (Device-Local)

On student's device (phone/browser):

```javascript
// Receive signing_data and signing_request_id from backend
// Use device biometric to authorize
// Sign with device key (NEVER transmitted raw)
// Return 128-char hex signature

const signature = await deviceSign(signing_data);
// signature: "abcdef0123456789..." (128 chars)
```

### Step 4: Submit Signature

```json
POST /api/v1/stellar/submit-signature

{
  "signing_request_id": "signing-uuid",
  "signature": "abcdef0123456789..." (128 hex chars)
}
```

Response:
```json
{
  "id": "auth-uuid",
  "payment_request_id": "payment-uuid",
  "authorized_by": "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
  "authorized_at": "2026-09-30T11:00:00Z"
}
```

### Step 5: Backend Processes Transaction

(Automatic in production)

- Backend verifies signature
- Creates transaction envelope
- Signs with environment-variable key
- Submits to Stellar testnet
- Polls for confirmation

### Step 6: Check Transaction Status

```json
GET /api/v1/stellar/transaction/tx-uuid
```

Response:
```json
{
  "id": "tx-uuid",
  "status": "confirmed",
  "tx_hash": "abcdef0123456789...",
  "ledger": 12345,
  "confirmed_at": "2026-09-30T11:01:00Z"
}
```

---

## Common Transactions

### Student Pays Merchant

```json
{
  "payer": "student-public-key",
  "payee": "merchant-public-key",
  "amount": "25.00",
  "payment_type": "student_to_merchant",
  "memo": "Food purchase"
}
```

### Merchant Refunds Student

```json
{
  "payer": "merchant-public-key",
  "payee": "student-public-key",
  "amount": "25.00",
  "payment_type": "merchant_to_student",
  "memo": "Refund for order"
}
```

### Student Sends to Student

```json
{
  "payer": "student1-key",
  "payee": "student2-key",
  "amount": "10.00",
  "payment_type": "student_to_student",
  "memo": "Lunch split"
}
```

---

## Query Transaction History

```bash
# Get all confirmed transactions for account
curl "http://localhost:8080/api/v1/stellar/transaction-history?account=GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW&status=confirmed&limit=50"

# Pagination
curl "http://localhost:8080/api/v1/stellar/transaction-history?account=...&limit=50&offset=50"
```

---

## Debugging

### Check Testnet Status

```bash
# Is testnet running?
curl https://soroban-testnet.stellar.org/health

# Check transaction on blockchain
# Go to: https://stellar.expert/explorer/testnet/tx/TX_HASH
```

### Check Database

```bash
# Connect to SQLite
sqlite3 data/id_card_system.db

# View payment transactions
SELECT * FROM payment_transactions ORDER BY created_at DESC LIMIT 5;

# View transaction history
SELECT * FROM payment_transaction_history ORDER BY timestamp DESC LIMIT 10;
```

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
```

### Check Environment

```bash
# Verify keys are set
echo $STELLAR_MASTER_SECRET_KEY
echo $STELLAR_MERCHANT_SECRET_KEY

# Verify URLs
echo $STELLAR_RPC_URL
echo $STELLAR_HORIZON_URL
```

---

## Testing

### Run All Tests

```bash
cargo test --lib stellar_payment
```

### Run Specific Test

```bash
cargo test --lib stellar_payment::tests::test_payment_request_creation
```

### Run With Output

```bash
cargo test --lib stellar_payment -- --nocapture
```

---

## Important Notes

### ⚠️ Security

- **NEVER** commit secret keys
- **NEVER** expose keys to frontend
- **NEVER** put keys in QR codes
- **ONLY** testnet keys in this system
- Device biometric processing is LOCAL

### 🔄 Transaction States

```
Payment Request:
  pending → authorized → completed ✓
        ↘ failed ✗

Payment Transaction:
  pending → submitted → confirmed ✓
       ↘ failed ✗ (retries up to 3x)
```

### ⏱️ Timeouts

- Payment request: 1 hour (configurable)
- Signing request: 5 minutes
- Transaction submission: 5 minutes before retry
- Max retries: 3 attempts (configurable)

### 💰 Limits

- Minimum: 0.1 XLM
- Maximum: 1000 XLM
- Fee: 100-10000 stroops (~0.001 XLM)

---

## Useful Stellar Tools

| Tool | URL | Purpose |
|------|-----|---------|
| Stellar Expert | https://stellar.expert | View testnet transactions |
| Friendbot | https://friendbot.stellar.org | Request testnet XLM |
| Horizon Docs | https://developers.stellar.org/docs | API reference |
| Soroban CLI | `soroban` command | Key management |

---

## Troubleshooting

### "Invalid network passphrase"
Check `.env` has correct network passphrase:
```
STELLAR_NETWORK_PASSPHRASE=Test SDF Network ; September 2015
```

### "Invalid public key"
Stellar keys must:
- Start with 'G' (public key)
- Be exactly 56 characters
- Be valid base32

### "Transaction timed out"
Check testnet connectivity and try again with `POST /api/v1/stellar/confirm-payment`

### "Insufficient funds"
Use Friendbot to request testnet XLM:
```bash
curl "https://friendbot.stellar.org/?addr=GXXXXXX"
```

---

## Next Steps

1. ✅ Setup complete
2. Test payment flow end-to-end
3. Review transaction on Stellar Expert
4. Integrate with frontend
5. Connect biometric authentication
6. Deploy to testnet
7. Monitor transaction history

---

**Happy testing! 🚀**

For full documentation, see `docs/STELLAR_PAYMENTS.md`
