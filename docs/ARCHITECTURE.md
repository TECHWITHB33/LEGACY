# Architecture Guide

## System Overview

The ID Card System is a multi-tier application designed for secure student identity management with blockchain integration and payment processing.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                 Frontend Layer                       │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  Streamlit   │  │    React     │  │   Mobile   │ │
│  │   (MVP)      │  │ (Production) │  │    App     │ │
│  └──────────────┘  └──────────────┘  └────────────┘ │
└────────────────────┬────────────────────────────────┘
                     │ HTTPS/REST
┌────────────────────▼────────────────────────────────┐
│                 API Gateway Layer                    │
│  - Rate Limiting, Auth, Validation, CORS            │
└────────────────────┬────────────────────────────────┘
                     │ HTTP/REST
┌────────────────────▼────────────────────────────────┐
│                  Service Layer (Rust)               │
│  ┌─────────────┐  ┌──────────┐  ┌──────────────┐   │
│  │ Auth Svc    │  │Credential│  │  Blockchain  │   │
│  └─────────────┘  └──────────┘  └──────────────┘   │
└────────────────────┬────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────┐
│              Data Layer                             │
│  ┌──────────────┐          ┌─────────────────────┐  │
│  │  PostgreSQL  │          │  Redis Cache        │  │
│  └──────────────┘          └─────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Component Details

### Frontend Layer

**Streamlit Frontend (MVP)**
- Rapid prototyping and validation
- User registration and login
- Credential viewing and QR generation

**React Frontend (Production)**
- Component-based architecture
- Real-time updates
- Offline capabilities

### Backend Services (Rust)

**Authentication Service**
- User registration and login
- JWT token generation
- Session management

**Credential Management**
- Lifecycle management
- Digital signature generation
- Versioning and status tracking

**Blockchain Service**
- Immutable records
- Proof-of-work validation
- Integrity verification

**Payment Service**
- Stellar integration
- Transaction tracking
- Refund handling

### Data Layer

**PostgreSQL Database**
- Primary data store
- ACID compliance
- Encryption at rest

**Redis Cache**
- Session caching
- Rate limiting
- Real-time data

## Technology Stack

| Layer | Component | Technology |
|-------|-----------|------------|
| Frontend | Web App | React + TypeScript |
| Frontend | MVP | Python + Streamlit |
| Backend | API Server | Rust + Actix-web |
| Backend | Auth | JWT + PBKDF2 |
| Backend | Crypto | RSA + EdDSA + AES |
| Backend | Blockchain | SHA-256 PoW |
| Data | Primary DB | PostgreSQL |
| Data | Cache | Redis |
| Infrastructure | Containers | Docker |
| Infrastructure | CI/CD | GitHub Actions |
