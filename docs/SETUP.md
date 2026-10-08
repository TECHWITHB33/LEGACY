# Development Setup Guide

## Prerequisites

### System Requirements
- **OS**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 20.04+)
- **RAM**: 8GB minimum, 16GB recommended
- **Disk**: 5GB free space
- **Network**: Internet connection for dependencies and Stellar testnet

### Required Software
- **Rust**: 1.70+ ([Install Rust](https://rustup.rs/))
- **Node.js**: 18+ LTS ([Install Node](https://nodejs.org/))
- **Python**: 3.9+ ([Install Python](https://www.python.org/downloads/))
- **Git**: Latest version
- **Docker**: 20.10+ (optional, for containerization)
- **PostgreSQL**: 15+ (optional, for production database)

## Installation Steps

### 1. Clone Repository
```bash
git clone https://github.com/yourusername/id-card-system.git
cd id-card-system
```

### 2. Backend Setup (Rust)

```bash
# Navigate to backend directory
cd backend

# Create .env file from example
cp .env.example .env

# Generate a secure JWT secret
# On Linux/macOS:
# echo "JWT_SECRET=$(openssl rand -hex 32)" >> .env
# On Windows PowerShell:
# Add-Content .env "JWT_SECRET=$(openssl rand -hex 32)"

# Install Rust dependencies
cargo build --release

# Run migrations
cargo sqlx migrate run

# Start development server
cargo run

# Or start with logging
RUST_LOG=debug cargo run
```

Backend will start on `http://localhost:8080`

### 3. Frontend Setup (Streamlit - MVP)

```bash
# Navigate to frontend directory
cd ../frontend_streamlit

# Create Python virtual environment
python -m venv venv

# Activate virtual environment
# On Linux/macOS:
# source venv/bin/activate
# On Windows:
# venv\Scripts\activate

# Install Python dependencies
pip install -r requirements.txt

# Create .env file
cp .env.example .env

# Start Streamlit app
streamlit run app.py
```

Streamlit will start on `http://localhost:8501`

### 4. Frontend Setup (React - Production)

```bash
# Navigate to React frontend directory
cd ../frontend_react

# Install Node dependencies
npm install

# Create .env file
echo "VITE_API_URL=http://localhost:8080/api/v1" > .env.local

# Start development server
npm run dev

# Build for production
npm run build
```

React will start on `http://localhost:5173`

## Configuration

### Backend Configuration (.env)
```env
# Database
DATABASE_URL=sqlite://data/id_card_system.db
LOG_LEVEL=debug

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# JWT
JWT_SECRET=your-32-byte-hex-secret-key-here
JWT_EXPIRY=3600

# Stellar (Testnet)
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE=Test SDF Network ; September 2015

# Features
ENABLE_BIOMETRIC_AUTH=true
ENABLE_STELLAR_PAYMENTS=true
ENABLE_QR_VERIFICATION=true
ENABLE_AUDIT_LOGGING=true
```

### Frontend Configuration (.env)
```
VITE_API_URL=http://localhost:8080/api/v1
VITE_STELLAR_NETWORK=testnet
VITE_LOG_LEVEL=debug
```

## Running Tests

### Backend Tests
```bash
cd backend

# Run all tests
cargo test

# Run specific test module
cargo test crypto::

# Run tests with output
cargo test -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test
```

### Frontend Tests (Streamlit)
```bash
cd frontend_streamlit

# Run pytest
pytest tests/

# Run with coverage
pytest --cov=src tests/
```

### Frontend Tests (React)
```bash
cd frontend_react

# Run Vitest
npm test

# Run with UI
npm run test:ui

# Run with coverage
npm test -- --coverage
```

## Database Setup

### SQLite (Development)
```bash
# Database is auto-created at: backend/data/id_card_system.db
# Migrations run automatically on startup
```

### PostgreSQL (Production)
```bash
# Create database
createdb id_card_system

# Update .env
DATABASE_URL=postgres://user:password@localhost:5432/id_card_system

# Run migrations
cd backend
cargo sqlx migrate run
```

## Stellar Testnet Setup

### Create Stellar Account
```bash
# Visit: https://laboratory.stellar.org
# Generate keypair
# Fund with testnet tokens
```

### Add to Backend
```env
STELLAR_ACCOUNT_ID=GXXXXXX...
STELLAR_SECRET_KEY=SXXXXXX...
```

## Docker Setup (Optional)

### Build Images
```bash
# Build backend
docker build -f docker/backend.Dockerfile -t id-card-backend:latest .

# Build frontend
docker build -f docker/frontend.Dockerfile -t id-card-frontend:latest .
```

### Run with Docker Compose
```bash
# Start all services
docker-compose -f docker-compose.yml up

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## Troubleshooting

### Backend Won't Start
```bash
# Check Rust installation
rustc --version

# Verify database connection
sqlite3 data/id_card_system.db ".schema"

# Check logs
RUST_LOG=debug cargo run
```

### Streamlit Connection Issues
```bash
# Verify backend is running
curl http://localhost:8080/health

# Check API URL in .env
cat frontend_streamlit/.env | grep API_BASE_URL

# Try direct API call
python -c "import requests; print(requests.get('http://localhost:8080/health').json())"
```

### React Build Issues
```bash
# Clear node modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Clear build cache
rm -rf dist
npm run build
```

### Database Errors
```bash
# Reset database (WARNING: deletes all data)
rm backend/data/id_card_system.db
cargo sqlx migrate run

# Check migrations
ls -la backend/migrations/
```

## Development Workflow

### Making Changes to Backend
```bash
# 1. Make code changes in src/
# 2. Run tests
cargo test

# 3. Format code
cargo fmt

# 4. Check for issues
cargo clippy

# 5. Rebuild
cargo build

# 6. Test with API
curl http://localhost:8080/health
```

### Making Changes to Frontend (Streamlit)
```bash
# 1. Edit app.py or components
# 2. Streamlit auto-reloads on save
# 3. Test in browser (reload page)
```

### Making Changes to Frontend (React)
```bash
# 1. Edit components in src/
# 2. Vite auto-reloads on save
# 3. View changes in browser (usually auto-refresh)
```

## Debugging

### Enable Verbose Logging
```bash
# Backend
RUST_LOG=trace cargo run

# Streamlit
streamlit run app.py --logger.level=debug

# React (in browser console)
localStorage.setItem('debug', 'id-card-*')
```

### API Testing with Curl
```bash
# Health check
curl http://localhost:8080/health

# Register user
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "testpass123",
    "name": "Test User",
    "institution": "Test University",
    "matric_number": "TEST001",
    "age": 20,
    "gender": "Male",
    "department": "CS",
    "phone_contact": "+2341234567890"
  }'

# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "testpass123"
  }'
```

### Database Inspection
```bash
# View users
sqlite3 data/id_card_system.db "SELECT id, email, status FROM users;"

# View credentials
sqlite3 data/id_card_system.db "SELECT id, user_id, status FROM credentials;"

# View blockchain
sqlite3 data/id_card_system.db "SELECT block_index, hash FROM blockchain_blocks ORDER BY block_index DESC LIMIT 10;"
```

## Next Steps

1. **Create test account** through UI
2. **Verify registration** in database
3. **Test authentication** with login
4. **Generate QR code** for credential
5. **Test blockchain** verification
6. **Create payment** on Stellar testnet
7. **Register biometric** credential
8. **Review audit logs**

## Getting Help

- **Documentation**: See `docs/` directory
- **API Reference**: See `docs/API.md`
- **Security Guide**: See `docs/SECURITY.md`
- **Issues**: Check GitHub issues or create new one
- **Discussions**: Use GitHub discussions for questions

## Performance Tips

- Use `--release` for production builds: `cargo build --release`
- Enable CORS caching in production
- Use Redis for session caching
- Implement database query optimization
- Monitor memory usage with system tools
- Use CDN for static frontend assets
