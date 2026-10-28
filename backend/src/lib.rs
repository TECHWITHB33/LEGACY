//! LEGACY Backend: Student Identity Card System with Invoice Liquidity Network
//!
//! This Rust backend provides:
//! - JWT-based authentication and authorization
//! - Cryptographic credential issuance and verification
//! - Soroban smart contract escrow for invoice liquidity
//! - Invoice lifecycle management (Draft → Submitted → Funded → Processing → Settled → Closed)
//! - Global advancement service for cross-border settlement with fee tracking
//! - Immutable audit logging for compliance

pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod jwt;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;
pub mod validators;

// New modules for invoice liquidity system
pub mod audit;
pub mod invoice_lifecycle;
pub mod global_advancement;
pub mod contracts;

// Existing modules
pub mod payment_gateway;
pub mod qr_verification;
pub mod stellar_payment;
pub mod biometric_auth;
pub mod credential_expiration;
pub mod credential_revocation;
pub mod credential_status;
pub mod credential_versioning;
pub mod crypto;
