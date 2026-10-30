# Vercel Multi-Service Deployment Guide

This document explains how to deploy the Legacy application to Vercel with multiple services.

## Architecture

The `vercel.json` configuration deploys three services:

1. **backend** (Rust/Actix-web)
   - Location: `./backend`
   - Runtime: Rust
   - Public Path: `/api/*` (via rewrite)
   - Internal: All API requests routed here

2. **frontend_react** (React/Vite)
   - Location: `./frontend_react`
   - Framework: Vite
   - Public Path: `/` (catch-all)
   - Bindings: Receives `VITE_API_URL` pointing to backend service
   - Public: Yes (serves the SPA)

## Prerequisites

1. **Vercel Account**: https://vercel.com
2. **GitHub Repository**: Push your code to GitHub
3. **Environment Variables**: Prepare secrets before deployment

## Step 1: Prepare Environment Variables

### Backend Variables (Required)

```
JWT_SECRET              = 64-character random hex string
DATABASE_URL            = sqlite:///data/id_card_system.db
STELLAR_NETWORK         = testnet
STELLAR_RPC_URL         = https://soroban-testnet.stellar.org
STELLAR_HORIZON_URL     = https://horizon-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE = Test SDF Network ; September 2015
STELLAR_MASTER_SECRET_KEY  = S... (testnet key from stellar.expert)
```

**Generate JWT_SECRET:**
```bash
openssl rand -hex 32
```

### Frontend Variables (Optional)

```
VITE_API_URL = Automatically injected by Vercel bindings
```

## Step 2: Deploy to Vercel

### Option A: Deploy via Vercel Dashboard (Recommended)

1. Go to https://vercel.com
2. Click "Add New..." → "Project"
3. Select your GitHub repository
4. Vercel will auto-detect `vercel.json`
5. Click "Environment Variables" and add:
   - `JWT_SECRET`
   - `DATABASE_URL`
   - `STELLAR_NETWORK`
   - `STELLAR_RPC_URL`
   - `STELLAR_HORIZON_URL`
   - `STELLAR_NETWORK_PASSPHRASE`
   - `STELLAR_MASTER_SECRET_KEY`
6. Click "Deploy"

### Option B: Deploy via Vercel CLI

```bash
# Install Vercel CLI
npm i -g vercel

# Login to Vercel
vercel login

# Deploy
vercel --prod

# Follow prompts to configure environment variables
```

## Step 3: Verify Deployment

### Check Services Are Running

```bash
# Frontend (public)
curl https://your-project.vercel.app

# Backend API (internal, but accessible via frontend)
curl https://your-project.vercel.app/api/v1/health

# Check binding injection
curl https://your-project.vercel.app/api/v1/system/info
```

### Local Testing with Vercel CLI

```bash
# Test multi-service deployment locally
vercel dev

# This will start:
# - Backend on internal port (auto-assigned)
# - Frontend on http://localhost:3000
# - Rewrites route /api to backend
# - Bindings inject VITE_API_URL pointing to backend
```

## Service Details

### Backend Service

**Build Command**: `cargo build --release`
**Start Command**: `./target/release/id-card-system`
**Port**: Internal (Vercel-managed)
**Routes**: `/api/*` routed via rewrite

**Key Routes:**
- `GET /api/v1/health` — Service health check
- `POST /api/v1/auth/login` — User login
- `GET /api/v1/escrow/invoices` — List invoices
- `POST /api/v1/escrow/invoices` — Create invoice
- `PUT /api/v1/escrow/invoices/{id}/fund` — Fund invoice (deploy escrow)
- `PUT /api/v1/escrow/invoices/{id}/settle` — Settle invoice

### Frontend Service

**Build Command**: `npm run build`
**Output Directory**: `dist`
**Bindings**: `VITE_API_URL` → backend service

**How Bindings Work:**
1. React app reads `import.meta.env.VITE_API_URL`
2. Vercel injects the backend service URL at runtime
3. Frontend API calls are automatically routed to backend

**Client Code Example:**
```typescript
import client from '@/utils/api'

// This uses the VITE_API_URL binding automatically
const invoices = await client.get('/escrow/invoices')
```

## Routing Logic

```
Request → Vercel Router → vercel.json rewrites
                             ↓
                        Check /api/(.*)?
                         YES ↓ NO ↓
                            backend   frontend_react
```

### Public Routes (Accessible from Internet)

- `/` → frontend_react (React SPA)
- `/api/*` → backend (Rust API)

### Internal Routes (Service-to-Service)

- Backend ← Frontend (via binding: `VITE_API_URL`)
- Backend is not publicly accessible without `/api` prefix

## Database Persistence

The SQLite database uses `DATABASE_URL=sqlite:///data/id_card_system.db`.

**⚠️ Important**: Vercel Functions don't persist filesystem state between deployments. For production:

1. **Option A**: Use a managed database (PostgreSQL, MySQL)
   ```
   DATABASE_URL=postgresql://user:pass@db.provider.com/legacy
   ```

2. **Option B**: Use Vercel KV for state (Redis-compatible)
   ```bash
   vercel env pull  # Pulls KV connection strings
   ```

3. **Option C**: Volume Storage (Vercel Pro feature)
   - Mounts persistent filesystem
   - Requires Vercel Pro plan

## Environment Variable Security

- ✅ Never commit `.env` files
- ✅ Never commit `STELLAR_MASTER_SECRET_KEY`
- ✅ Use Vercel dashboard for secrets
- ✅ Rotate secrets periodically
- ✅ Use separate keys for testnet/mainnet

## Troubleshooting

### Backend Build Fails: MSVC Linker Not Found

**Windows/Vercel Issue**: The default Linux builder may not have Rust.

**Solution**: Add `engines` to `backend/package.json`:
```json
{
  "engines": {
    "node": "18.x"
  }
}
```

Or specify a custom build in `vercel.json`:
```json
{
  "buildCommand": "cd backend && rustup target add x86_64-unknown-linux-gnu && cargo build --target x86_64-unknown-linux-gnu --release"
}
```

### Frontend Can't Connect to Backend

**Check**: Verify `VITE_API_URL` binding is injected

```bash
# In frontend code, log the URL
console.log('API URL:', import.meta.env.VITE_API_URL)

# Should show the backend service URL
```

**Fix**: Ensure binding is declared in `vercel.json`:
```json
{
  "bindings": [
    {
      "type": "service",
      "service": "backend",
      "format": "url",
      "env": "VITE_API_URL"
    }
  ]
}
```

### Database Errors: Not Found

**Issue**: SQLite database file not persisting

**Temporary Solution** (Testing): Store in `/tmp`
```
DATABASE_URL=sqlite:////tmp/id_card_system.db
```

**Production Solution**: Migrate to PostgreSQL
```
DATABASE_URL=postgresql://user:pass@neon.tech/legacy
```

## Monitoring & Logs

### View Logs in Vercel Dashboard

1. Go to https://vercel.com/dashboard
2. Select your project
3. Click "Deployments"
4. Select a deployment
5. Click "Logs"

### View Live Logs

```bash
vercel logs --prod
```

## Rollback

If deployment breaks, rollback quickly:

```bash
vercel rollback
```

Or via dashboard:
1. Deployments tab
2. Select previous deployment
3. Click "Promote to Production"

## Scaling & Performance

### Backend Optimization

- Rust binary is pre-compiled; minimal startup time
- Actix-web handles concurrent connections efficiently
- Soroban contract calls are async/non-blocking

### Frontend Optimization

- Vite produces optimized dist bundle
- React Router enables code splitting
- TailwindCSS provides minimal CSS

### Database

- For high traffic: Consider connection pooling
- For read-heavy: Add Redis cache layer
- For write-heavy: Migrate to managed database

## Local Development

```bash
# Install dependencies
npm install

# Backend
cd backend
cargo build

# Run locally
cargo run

# Frontend
cd ../frontend_react
npm install
npm run dev

# Or test with Vercel CLI
vercel dev
```

## Cost Estimate

- **Vercel Hobby (Free)**: Up to 3 deployments/week
- **Vercel Pro ($20/month)**: Unlimited deployments + volume storage
- **Database**: ~$15-50/month if migrating from SQLite

## Next Steps

1. ✅ Push code to GitHub
2. ✅ Create Vercel account
3. ✅ Connect GitHub repository
4. ✅ Add environment variables
5. ✅ Deploy
6. ✅ Test endpoints
7. ✅ Monitor logs
8. ✅ Set up custom domain (optional)

## Support

- Vercel Docs: https://vercel.com/docs
- API Routes: https://vercel.com/docs/functions/serverless-functions
- Environment Variables: https://vercel.com/docs/concepts/projects/environment-variables
- Soroban: https://soroban.stellar.org/docs

---

**Last Updated**: October 2026
**Status**: Ready for Production Deployment
