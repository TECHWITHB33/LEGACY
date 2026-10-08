# 🎓 Student Identity Card Security System

[![CI Status](https://github.com/yourusername/id-card-system/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/id-card-system/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.70+](https://img.shields.io/badge/Rust-1.70%2B-orange)](https://www.rust-lang.org/)

> **Enhanced Student Identity Card Security for Education Sector**
> 
> Blockchain-Based Secure Identity Card System with QR Code Verification, Biometric Authentication, and Stellar Payment Integration

## 🌟 Features

### Identity Management
- ✅ Student registration with cryptographic identity
- ✅ RSA 4096-bit keypair generation
- ✅ EdDSA digital signatures for authentication
- ✅ Public key infrastructure (PKI)
- ✅ Credential lifecycle management

### Security
- ✅ SHA256 hashing for data integrity
- ✅ PBKDF2-SHA256 password key derivation (100k iterations)
- ✅ Encrypted private key storage (AES-256-GCM)
- ✅ JWT token-based authentication
- ✅ Rate limiting and abuse prevention
- ✅ Comprehensive audit logging

### Blockchain
- ✅ SHA256-based blockchain with proof-of-work
- ✅ Immutable credential records
- ✅ Chain integrity validation
- ✅ Tamper detection

### QR Codes
- ✅ QR code generation for credentials
- ✅ QR code verification and scanning
- ✅ Embedded credential data
- ✅ Device-independent verification

### Biometric Authentication
- ✅ FIDO2/WebAuthn support
- ✅ Fingerprint recognition
- ✅ Face recognition
- ✅ Device-bound authentication

### Payment Processing
- ✅ Stellar testnet integration
- ✅ Student payment authorization
- ✅ Merchant payment QR codes
- ✅ Payment verification and tracking
- ✅ Transaction history

### Admin & Compliance
- ✅ Full audit trail
- ✅ User management dashboard
- ✅ System statistics
- ✅ GDPR compliance ready
- ✅ SOC 2 alignment

## 📋 Prerequisites

### System Requirements
- **OS**: Windows 10+, macOS 10.15+, Linux (Ubuntu 20.04+)
- **RAM**: 8GB minimum, 16GB recommended
- **Disk**: 5GB free space
- **Docker**: 20.10+ (optional, recommended)

### Software
- **Rust**: 1.70+ ([Install](https://rustup.rs/))
- **Node.js**: 18+ LTS ([Install](https://nodejs.org/))
- **Python**: 3.9+ ([Install](https://www.python.org/))
- **Git**: Latest version

## 🚀 Quick Start

### Using Docker (Recommended)

```bash
# Clone repository
git clone https://github.com/yourusername/id-card-system.git
cd id-card-system

# Create .env from example
cp backend/.env.example backend/.env

# Generate JWT secret
echo "JWT_SECRET=$(openssl rand -hex 32)" >> backend/.env

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f
```

Services will be available at:
- **Backend API**: http://localhost:8080
- **Streamlit Frontend**: http://localhost:8501
- **React Frontend**: http://localhost:3000

### Local Development

#### Backend Setup (Rust)
```bash
cd backend

# Create environment file
cp .env.example .env

# Install dependencies and build
cargo build --release

# Run migrations
cargo sqlx migrate run

# Start server
cargo run
```

Backend at: `http://localhost:8080`

#### Frontend Setup (Streamlit)
```bash
cd frontend_streamlit

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Create .env
echo "API_BASE_URL=http://localhost:8080/api/v1" > .env

# Start frontend
streamlit run app.py
```

Frontend at: `http://localhost:8501`

#### Frontend Setup (React)
```bash
cd frontend_react

# Install dependencies
npm install

# Create environment file
echo "VITE_API_URL=http://localhost:8080/api/v1" > .env.local

# Start development server
npm run dev
```

Frontend at: `http://localhost:5173`

## 📚 Documentation

- **[Architecture Guide](docs/ARCHITECTURE.md)** - System design and data flow
- **[Security Guide](docs/SECURITY.md)** - Cryptography and security measures
- **[API Reference](docs/API.md)** - Complete API documentation
- **[Setup Guide](docs/SETUP.md)** - Development environment setup
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Production deployment

## 🔐 Security

This system implements enterprise-grade security:

### Cryptography
- **RSA 4096-bit**: User key management
- **EdDSA**: Digital signatures
- **SHA256**: Hashing
- **PBKDF2**: Password derivation
- **AES-256-GCM**: Encryption at rest

### Authentication
- **JWT tokens**: Short-lived (1 hour)
- **Refresh tokens**: Long-lived (30 days)
- **Rate limiting**: Brute force prevention
- **Session management**: Secure HTTP-only cookies

### Compliance
- ✅ OWASP Top 10 protections
- ✅ GDPR data protection
- ✅ SOC 2 audit logging
- ✅ PCI DSS for payments

See [Security Guide](docs/SECURITY.md) for details.

## 🧪 Testing

### Run Tests
```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd ../frontend_streamlit
pytest tests/

# React tests
cd ../frontend_react
npm test
```

### Test Coverage
```bash
# Backend coverage
cargo tarpaulin --out Html

# Python coverage
pytest --cov=src tests/
```

## 📊 API Examples

### Register User
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "student@university.edu",
    "password": "SecurePassword123!",
    "name": "John Doe",
    "institution": "Federal University of Technology Minna",
    "matric_number": "MTN/SCI/001/2023",
    "age": 20,
    "gender": "Male",
    "department": "Computer Science",
    "phone_contact": "+2341234567890"
  }'
```

### Login
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "student@university.edu",
    "password": "SecurePassword123!"
  }'
```

### Get Blockchain
```bash
curl http://localhost:8080/api/v1/blockchain/chain \
  -H "Authorization: Bearer {token}"
```

More examples in [API Reference](docs/API.md).

## 🚢 Deployment

### Production Deployment

1. **Environment Setup**
   ```bash
   # Set strong JWT secret
   export JWT_SECRET=$(openssl rand -hex 32)
   
   # Configure database
   export DATABASE_URL=postgresql://user:pass@host/db
   
   # Set Stellar credentials
   export STELLAR_ACCOUNT_ID=GXXXXX...
   export STELLAR_SECRET_KEY=SXXXXX...
   ```

2. **Build Images**
   ```bash
   docker build -f docker/backend.Dockerfile -t id-card-backend:latest .
   docker build -f docker/frontend.Dockerfile -t id-card-frontend:latest .
   ```

3. **Deploy to Kubernetes**
   ```bash
   kubectl apply -f k8s/
   ```

4. **Enable HTTPS**
   - Install SSL certificate
   - Configure reverse proxy (Nginx/HAProxy)
   - Enable HSTS headers

See [Deployment Guide](docs/DEPLOYMENT.md) for detailed instructions.

## 📦 Project Structure

```
.
├── backend/              # Rust API server
├── frontend_streamlit/   # Streamlit MVP
├── frontend_react/       # React production UI
├── docker/               # Docker configurations
├── .github/workflows/    # CI/CD pipelines
├── docs/                 # Documentation
└── .kiro/steering/       # Development guidelines
```

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📝 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

## ⚠️ Security Disclaimer

This system handles sensitive student identity and payment data. Before deploying to production:

- [ ] Conduct security audit
- [ ] Penetration test the system
- [ ] Review cryptographic implementation
- [ ] Validate compliance requirements
- [ ] Set up monitoring and alerting
- [ ] Establish incident response procedures

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/jc1788-byte/id-card-system/issues)
- **Discussions**: [GitHub Discussions](https://github.com/jc1788-byte/id-card-system/discussions)
- **Email**:jessicakes1788@gmail.com
- **Documentation**: [[docs/](docs/)](https://github.com/jc1788-byte)

## 🎯 Roadmap

### v1.1 (Q2 2024)
- [ ] Webhook support for real-time events
- [ ] Mobile app (iOS/Android)
- [ ] Advanced analytics dashboard
- [ ] Multi-language support

### v1.2 (Q3 2024)
- [ ] Integration with university systems
- [ ] Batch import for students
- [ ] Advanced reporting
- [ ] API rate limiting improvements

### v2.0 (Q4 2024)
- [ ] Ethereum smart contracts
- [ ] Multi-chain support
- [ ] Advanced ML-based fraud detection
- [ ] Hardware security module (HSM) support



## 📞 Contact

- **github**:[ https://yourdomain.com](https://github.com/jc1788-byte)
- **Email**: jessicakes1788@gmail.com


## 🙏 Acknowledgments

- Stellar Development Foundation (Stellar network)
- FIDO Alliance (WebAuthn standard)
- Rust community
- Open source contributors

---

Made with ❤️ for secure student identity management (OLAREWAJU.SAMUEL.OLUSEGUN)
