# Implementation Plan: Soroban Escrow & Invoice Liquidity Network Integration

## Overview

This plan integrates a modernized Soroban smart contract system for invoice liquidity on Stellar into the existing LEGACY student identity card system. The work decomposes into four features (FEAT-001 through FEAT-004), each delivering independent, testable value while building toward a complete invoice-to-settlement pipeline.

**Project Context**: Backend Rust/Actix-web system with SQLite, existing Stellar testnet integration, established patterns for services, handlers, and error handling. New integration adds: Soroban escrow contracts, invoice lifecycle state machine (Draft → Submitted → Funded → Processing → Settled → Closed), cross-border settlement with fee tracking, and REST API endpoints.

---

## FEAT-001: Database Schema & Data Models

**Description**: Create database schema for invoices, escrow contracts, state history, and settlement fees. Define Rust models reflecting immutable, audit-logged state.

**Design Decision**: Use immutable-by-design pattern—once invoice/escrow created, never UPDATE core fields; state changes recorded as new history entries. This ensures tamper-proof settlement records and simplifies audit trails.

**Files to Create/Modify**:
- `backend/migrations/20240104000003_invoice_escrow.sql` — Create 4 tables: invoices, escrow_contracts, escrow_state_history, settlement_fees. All with indexes on foreign keys and timestamps.
- `backend/src/models.rs` — Add: Invoice, Escrow, EscrowStateHistory, SettlementFee, FeeBreakdown structs with sqlx::FromRow and serde derives. Add InvoiceState enum (Draft, Submitted, Funded, Processing, Settled, Closed). Add request/response DTOs.

**Verification**:
```bash
cargo check                    # Verify compilation
sqlx migrate info             # Verify migration present
cargo build                   # Full build
grep -E "struct (Invoice|Escrow)" backend/src/models.rs  # Confirm structs exist
```

---

## FEAT-002: Soroban Contract Spec & RPC Layer

**Description**: Define Soroban escrow contract specification and implement RPC interaction layer to deploy and invoke contract methods on Stellar testnet.

**Design Decision**: 
- Contract spec documented as Rust comments; actual contract implementation (Rust/WASM) would be separate and deployed separately.
- RPC layer uses soroban-client crate (already in Cargo.toml 0.5); wraps connection, transaction signing, and error handling.
- No secret keys in source; all signing via server-managed keys from environment.
- Testnet only—RPC endpoint from environment variable or hardcoded testnet.

**Files to Create/Modify**:
- `backend/src/contracts/mod.rs` — Module declaration.
- `backend/src/contracts/escrow.rs` — SorobanEscrowSpec documenting: methods (deposit_funds, release_funds, refund_funds, get_state), events (FundsDeposited, FundsReleased, FundsRefunded, StateChanged), stored data (invoice_id, investor_account, borrower_account, amount_stroops, expiry_timestamp, settled flag, refunded flag).
- `backend/src/services/escrow_service.rs` — EscrowService with: deploy_escrow_contract(), invoke_deposit_funds(), invoke_release_funds(), get_contract_state(), validate_contract_deployment(). SorobanRpcClient helper for connection and RPC calls.
- `backend/src/services/mod.rs` — Export EscrowService.
- `backend/Cargo.toml` — Verify soroban-sdk ≥21, stellar-xdr ≥21, soroban-client ≥0.5 versions.

**Verification**:
```bash
cargo check
cargo clippy                  # No warnings on new code
cargo build
grep "SorobanEscrowSpec" backend/src/contracts/escrow.rs
grep "impl EscrowService" backend/src/services/escrow_service.rs
grep "soroban_sdk\|stellar_xdr\|soroban_client" backend/Cargo.toml
```

---

## FEAT-003: Invoice Lifecycle & Global Advancement

**Description**: Implement invoice lifecycle state machine enforcing valid transitions, and global advancement service coordinating multi-currency settlement with fee tracking.

**Design Decision**:
- Lifecycle is strict state machine; invalid transitions rejected with ValidationError.
- Each state change logged to audit trail and escrow_state_history table.
- Fund transition triggers Soroban contract deployment via EscrowService.
- Global advancement orchestrates settlement: fetch invoice + escrow, calculate fees, convert currencies if needed, record fees immutably, coordinate with contract release.
- Fees stored immutably; each fee type (platform, network, conversion) separate row in settlement_fees table.

**Files to Create/Modify**:
- `backend/src/invoice_lifecycle.rs` — InvoiceLifecycleService with methods: create_invoice(), submit_invoice(), fund_invoice(), mark_processing(), settle_invoice(), close_invoice(), refund_invoice(). Each method validates state, updates database immutably (INSERT for history), logs to audit, handles errors as ApiError.
- `backend/src/global_advancement.rs` — GlobalAdvancementService with: calculate_fees() (compute platform 1%, network, conversion), record_settlement_fees() (INSERT into settlement_fees), convert_asset() (lookup/fetch exchange rate), execute_global_advancement() (orchestrate full settlement).
- `backend/src/models.rs` — Add FeeBreakdown struct if not in FEAT-001.
- Unit tests in each service file covering: valid transitions, invalid rejections, fee calculations, currency conversions, immutability.

**Verification**:
```bash
cargo check
cargo test --lib invoice_lifecycle -- --nocapture
cargo test --lib global_advancement -- --nocapture
cargo build
grep "async fn.*invoice\|async fn.*settlement" backend/src/invoice_lifecycle.rs
grep "calculate_fees\|record_settlement_fees" backend/src/global_advancement.rs
```

---

## FEAT-004: REST API Handlers & Routing

**Description**: Implement REST endpoints for invoice and escrow operations, wire routes with JWT middleware, ensure fail-closed authorization.

**Design Decision**:
- Routes under /api/v1/escrow and /api/v1/invoices, all protected by JwtMiddleware.
- Authorization: borrower can only see their own invoices; investor can see invoices they funded; admin can settle.
- Handlers extract user/role from JWT token, validate ownership before operation, fail-closed (reject by default).
- Responses follow existing ApiError/ErrorResponse pattern.

**Files to Create/Modify**:
- `backend/src/handlers/escrow.rs` — Handler functions: create_invoice(), submit_invoice(), fund_invoice(), get_invoice(), list_invoices(), settle_invoice(), get_escrow_status(). Each verifies authorization, calls appropriate service, returns HttpResponse.
- `backend/src/handlers/mod.rs` — Add `pub mod escrow;` export.
- `backend/src/main.rs` — Add /api/v1/escrow scope with routes, all wrapped in JwtMiddleware.
- Integration tests covering: full flow (create → submit → fund → settle → close), authorization checks, error cases.

**Verification**:
```bash
cargo check
cargo build
cargo test --lib handlers::escrow -- --nocapture
cargo test --test '*' -- --nocapture  # if integration tests present
grep "/api/v1/escrow" backend/src/main.rs
grep "async fn.*invoice\|async fn.*escrow" backend/src/handlers/escrow.rs
cargo run &  # Start server
curl -H "Authorization: Bearer <JWT>" http://localhost:8080/api/v1/escrow/invoices
```

---

## Implementation Sequence

**Order is strict—each feature depends on prior ones**:

1. **FEAT-001** (Database & Models): No dependencies; establishes data layer.
2. **FEAT-002** (Soroban & RPC): Depends on FEAT-001 models; implements contract layer.
3. **FEAT-003** (Lifecycle & Advancement): Depends on FEAT-001 (models) and FEAT-002 (escrow service); implements business logic.
4. **FEAT-004** (API & Routing): Depends on all prior; exposes business logic via REST.

---

## Cross-Cutting Concerns

### Error Handling
All new code maps errors to ApiError variants:
- Contract deployment failures → `ApiError::StellarError`
- Invalid state transitions → `ApiError::ValidationError`
- Database errors → `ApiError::DatabaseError` (mapped automatically by sqlx)
- Authorization failures → `ApiError::AuthorizationError`

### Audit Logging
Every state change logged via `audit::log_event()` with:
- user_id (from JWT)
- AuditEvent (new: PaymentSubmitted, PaymentAuthorized, etc.)
- resource_type (e.g., "invoice", "escrow")
- resource_id (invoice_id, contract_id)
- details (JSON with state, amounts, metadata)

### Immutability
- Never UPDATE invoice.state, escrow.amount, escrow.settlement_expiry after creation.
- Record state changes in escrow_state_history table (INSERT only).
- Record fees in settlement_fees table (INSERT only).
- Enables tamper-proof audit trail and compliance.

### Testing
Each feature includes unit tests for:
- Happy paths (normal operations)
- Error paths (validation, authorization, network failures)
- Boundary conditions (zero amounts, expired timestamps, unknown states)

Run all tests: `cargo test --lib`

---

## Cargo.toml Dependencies (Verify/Add as Needed)

Current versions per exploration:
```toml
soroban-client = "0.5"        # ✓ Present
stellar-xdr = { version = "21", features = ["std", "serde", "base64"] }  # ✓ Present
stellar-strkey = "0.0.8"      # ✓ Present
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "sqlite", "uuid", "chrono"] }  # ✓ Present
serde = { version = "1.0", features = ["derive"] }  # ✓ Present
chrono = { version = "0.4", features = ["serde"] }  # ✓ Present
uuid = { version = "1.6", features = ["v4", "serde"] }  # ✓ Present
tracing = "0.1"              # ✓ Present
```

Possible additions (verify compatibility):
```toml
# If contract tests needed locally:
soroban-sdk = "21.0"          # For contract types/testing (optional, separate crate may be used)
```

---

## Environment Variables Required

New environment variables (add to `.env`):
```bash
SOROBAN_RPC_ENDPOINT=https://soroban-testnet.stellar.org/  # Testnet RPC
STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"  # Testnet
# Existing:
DATABASE_URL=sqlite://data/id_card_system.db
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
JWT_SECRET=<random-32-byte-hex>
```

---

## Deliverables Summary

| Artifact | Purpose |
|----------|---------|
| FEAT-001.json | Schema & models plan |
| FEAT-002.json | Soroban & RPC plan |
| FEAT-003.json | Lifecycle & advancement plan |
| FEAT-004.json | API & routing plan |
| context.json | Codebase context for implementers |
| task.json | Task metadata & ordering |
| plan.md | This document |

Each FEAT is independently reviewable, testable, and committable. Features are sequential; no parallel work.

---

## Success Criteria (Full Integration)

- [ ] All 4 FEATs implemented and passing tests
- [ ] `cargo build --release` produces working binary
- [ ] `cargo test --lib` passes all unit tests
- [ ] Endpoints accessible and functioning with valid JWTs
- [ ] Full invoice lifecycle testable: create → submit → fund (escrow deployed) → process → settle (fees recorded) → close
- [ ] Audit logs show every state transition
- [ ] Settlement fees immutable in database
- [ ] No secret keys in source, logs, or responses
- [ ] Soroban testnet only (no mainnet possibility)

