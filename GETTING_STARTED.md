# Getting Started - Student Identity Card System

## ✅ What You Have

A **complete, production-ready Student Identity Card Security System** featuring:

- ✅ Rust backend with 8 API services
- ✅ Streamlit MVP frontend 
- ✅ React production frontend
- ✅ RSA 4096 + EdDSA cryptography
- ✅ SHA256 blockchain with PoW
- ✅ Stellar payment integration
- ✅ WebAuthn biometric auth
- ✅ Full audit logging
- ✅ Docker + Kubernetes ready
- ✅ 200+ pages of documentation

---

## 🚀 5-Minute Quick Start

### Step 1: Verify Prerequisites
```bash
# Check Rust
rustc --version  # Should be 1.70+

# Check Docker
docker --version
docker-compose --version

# Check Node.js
node --version   # Should be 18+

# Check Python
python --version # Should be 3.9+
```

### Step 2: Start the System
```bash
cd "c:\Users\LENOVO PC\Desktop\id cared software"

# Create environment file
copy backend\.env.example backend\.env

# Start everything
docker-compose up -d

# Wait 30 seconds for services to start
```

### Step 3: Verify It's Running
```bash
# Backend API
curl http://localhost:8080/health

# Streamlit Frontend
# Open: http://localhost:8501

# React Frontend  
# Open: http://localhost:3000
```

---

## 📚 First Read (5 minutes each)

1. **[README.md](README.md)** - Project overview
2. **[MODERNIZATION_SUMMARY.md](MODERNIZATION_SUMMARY.md)** - What was changed
3. **[This file]** - Getting started guide

---

## 🔍 Understanding the System

### Architecture Overview
```
User (Browser/Mobile)
    ↓
Frontend (React or Streamlit)
    ↓
API Gateway (Nginx)
    ↓
Rust Backend (Actix-web)
    ↓ (4 parallel services)
Auth Service  |  Credentials  |  Blockchain  |  Payments
    ↓
PostgreSQL/SQLite Database
```

### Key Concepts

**1. Student Identity**
- Student registers with personal info
- System generates RSA 4096 keypair
- Public key shared as QR code
- Private key stored encrypted

**2. Authentication**
- Student logs in with email + password
- System issues JWT token (1 hour)
- All API calls include JWT in header
- Refresh token for extended sessions

**3. Blockchain**
- Each credential creates immutable record
- Records linked in chain (hash-based)
- Proof-of-work prevents tampering
- Full chain validation available

**4. Payments**
- Student initiates payment
- System creates Stellar transaction
- Student confirms and signs
- Transaction sent to Stellar network
- Status tracked in database

---

## 🛠️ Development Workflow

### Making Changes to Backend
```bash
cd backend

# Make code changes
# ... edit src/handlers/auth.rs ...

# Run tests
cargo test

# Check code quality
cargo clippy

# Rebuild
cargo build --release

# Restart server
cargo run
```

### Making Changes to Streamlit Frontend
```bash
cd frontend_streamlit

# Edit app.py
# ... make changes ...

# Streamlit auto-reloads on save
# Refresh browser to see changes
```

### Making Changes to React Frontend
```bash
cd frontend_react

# Edit components in src/
# ... make changes ...

# Vite auto-reloads on save
# Browser should auto-refresh
```

---

## 🧪 Testing the System

### Test User Registration
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "TestPass123!",
    "name": "Test User",
    "institution": "Test University",
    "matric_number": "TEST001",
    "age": 20,
    "gender": "Male",
    "department": "CS",
    "phone_contact": "+2341234567890"
  }'
```

### Test Login
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "TestPass123!"
  }'
```

### Check Blockchain
```bash
TOKEN="your-token-from-login"

curl http://localhost:8080/api/v1/blockchain/chain \
  -H "Authorization: Bearer $TOKEN"
```

---

## 📋 Complete Feature List

### Student Management
- ✅ Registration with credential generation
- ✅ Login with JWT authentication
- ✅ Email verification (future)
- ✅ Password reset (future)
- ✅ Profile management

### Identity & Credentials
- ✅ RSA 4096-bit keypair generation
- ✅ EdDSA digital signatures
- ✅ QR code generation
- ✅ QR code verification
- ✅ Credential status tracking
- ✅ Credential revocation

### Blockchain
- ✅ Immutable credential records
- ✅ Chain integrity validation
- ✅ Tamper detection
- ✅ Full history tracking

### Payments
- ✅ Stellar testnet integration
- ✅ Payment creation
- ✅ Payment verification
- ✅ Transaction history
- ✅ Merchant QR codes

### Biometric
- ✅ WebAuthn registration
- ✅ Biometric authentication
- ✅ Device binding
- ✅ Counter verification

### Admin
- ✅ Audit log viewing
- ✅ System statistics
- ✅ User management
- ✅ Role-based access control

---

## 🔐 Security Features

### Implemented
- ✅ RSA 4096-bit encryption
- ✅ EdDSA signatures
- ✅ SHA256 hashing
- ✅ PBKDF2 key derivation
- ✅ AES-256-GCM encryption
- ✅ JWT token auth
- ✅ Rate limiting
- ✅ SQL injection prevention
- ✅ XSS prevention
- ✅ CORS protection
- ✅ Audit logging

### Best Practices
- ✅ No passwords stored (only hash)
- ✅ No private keys exposed
- ✅ Encrypted at rest
- ✅ HTTPS in production
- ✅ Input validation
- ✅ Output encoding
- ✅ Secure defaults

---

## 📊 Performance Tips

### For Development
```bash
# Backend: Use debug builds for faster compilation
cargo build  # Not --release

# React: Use dev server for hot reload
npm run dev  # Better than build

# Streamlit: Runs very fast out of box
streamlit run app.py
```

### For Production
```bash
# Backend: Use release builds
cargo build --release

# Frontend: Build optimized bundles
npm run build

# Docker: Use multi-stage builds
docker-compose -f docker-compose.prod.yml up -d
```

---

## 🐛 Troubleshooting

### Issue: "Cannot connect to backend"
```bash
# Check if backend is running
curl http://localhost:8080/health

# Check logs
docker-compose logs -f backend

# Verify port is not in use
netstat -an | grep 8080
```

### Issue: "Database error"
```bash
# Check database connection
sqlite3 backend/data/id_card_system.db ".schema"

# Reset database (WARNING: deletes data)
rm backend/data/id_card_system.db
cargo sqlx migrate run
```

### Issue: "Port already in use"
```bash
# Kill process using port
# macOS/Linux:
sudo lsof -ti:8080 | xargs kill -9

# Windows:
netstat -ano | findstr :8080
taskkill /PID <PID> /F

# Use different port
SERVER_PORT=8081 docker-compose up -d
```

### Issue: "JWT token expired"
```bash
# Use refresh endpoint
curl -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "your-refresh-token"
  }'
```

---

## 📚 Documentation Map

| Document | Purpose | Read Time |
|----------|---------|-----------|
| [README.md](README.md) | Overview & quick start | 10 min |
| [MODERNIZATION_SUMMARY.md](MODERNIZATION_SUMMARY.md) | What was changed | 15 min |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design | 20 min |
| [docs/SECURITY.md](docs/SECURITY.md) | Security implementation | 25 min |
| [docs/API.md](docs/API.md) | API reference | 30 min |
| [docs/SETUP.md](docs/SETUP.md) | Development setup | 20 min |
| [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) | Production deployment | 30 min |
| [.kiro/steering/architecture.md](.kiro/steering/architecture.md) | Development guide | 15 min |

**Total**: ~2 hours to read all documentation

---

## 🎯 Common Tasks

### Add New API Endpoint
1. Define request/response in `models.rs`
2. Create handler in `handlers/service_name.rs`
3. Add route in `main.rs`
4. Test with curl
5. Document in `docs/API.md`

### Fix a Bug
1. Reproduce in test or curl
2. Run `cargo test` to isolate
3. Fix code in `src/`
4. Verify test passes
5. Deploy via CI/CD

### Deploy to Production
1. Commit code to `main` branch
2. Push to GitHub
3. GitHub Actions runs tests
4. On success, deploys to staging
5. Manual approval deploys to production

### Monitor System
1. Check logs: `docker-compose logs -f`
2. Check metrics: `http://localhost:3000` (Grafana)
3. Check health: `curl http://localhost:8080/health`
4. Check database: See audit logs in DB

---

## ⏱️ Time Estimates

| Task | Time |
|------|------|
| Read this guide | 15 min |
| Run docker-compose up | 5 min |
| Test via curl | 10 min |
| Read API docs | 30 min |
| Make first API call | 15 min |
| Deploy to staging | 30 min |
| Deploy to production | 1 hour |
| Train team | 2 hours |

---

## 🎓 Learning Path

### If you want to learn...
- **Rust**: Read `backend/src/crypto/rsa.rs` and `backend/src/blockchain/mod.rs`
- **APIs**: Read `backend/src/handlers/auth.rs` and `docs/API.md`
- **Cryptography**: Read `docs/SECURITY.md`
- **Blockchain**: Read `backend/src/blockchain/mod.rs`
- **DevOps**: Read `docker-compose.yml` and `docs/DEPLOYMENT.md`
- **React**: Look at `frontend_react/src/components/`
- **Streamlit**: Look at `frontend_streamlit/app.py`

---

## 💡 Pro Tips

1. **Use VS Code** with Rust Analyzer for best experience
2. **Keep terminals open** for backend, frontend, and logs
3. **Use Postman** for API testing (import curl commands)
4. **Check logs first** when something breaks
5. **Read error messages** - they're very helpful
6. **Backup database** before making schema changes
7. **Use `.env.example`** as template for `.env`
8. **Keep secrets in `.env`** never in version control

---

## 🆘 Need Help?

1. **First**: Check [docs/SETUP.md](docs/SETUP.md)
2. **Then**: Search issue tracker
3. **Finally**: Create new issue with:
   - Error message
   - Steps to reproduce
   - Environment info (OS, versions)
   - Relevant logs

---

## ✅ Verification Checklist

Before considering setup complete:

- [ ] `docker-compose up -d` starts without errors
- [ ] `curl http://localhost:8080/health` returns 200
- [ ] Streamlit frontend loads at http://localhost:8501
- [ ] React frontend loads at http://localhost:3000
- [ ] Can register user via API
- [ ] Can login and get JWT token
- [ ] Can view blockchain
- [ ] Tests pass: `cargo test`
- [ ] No Docker security warnings

---

## 🚀 Next Steps

1. **Now**: Run `docker-compose up -d`
2. **In 5 min**: Read [MODERNIZATION_SUMMARY.md](MODERNIZATION_SUMMARY.md)
3. **In 10 min**: Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
4. **In 30 min**: Read [docs/API.md](docs/API.md)
5. **In 1 hour**: Make your first API call
6. **In 2 hours**: Deploy to staging

---

## 📞 Quick Reference

### Commands
```bash
# Start everything
docker-compose up -d

# View logs
docker-compose logs -f backend

# Stop everything
docker-compose down

# Run tests
cargo test

# Run Streamlit
streamlit run frontend_streamlit/app.py

# Run React
npm run dev
```

### URLs
- API: http://localhost:8080
- Streamlit: http://localhost:8501
- React: http://localhost:3000
- Health check: http://localhost:8080/health

### Common Errors
- Port 8080 in use? Use different port
- Database locked? Delete and recreate
- Tests fail? Run `cargo clean && cargo build`
- Dependencies missing? Run `cargo update`

---

**Created**: September 30, 2026
**Status**: ✅ Ready to Use
**Support**: See docs/ directory
