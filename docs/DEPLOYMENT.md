# Production Deployment Guide

## Prerequisites

### Infrastructure Requirements
- **Compute**: 2+ CPU cores, 4GB+ RAM
- **Storage**: 50GB+ SSD
- **Network**: Public IP with static DNS
- **Backup**: Daily automated backups
- **Monitoring**: Log aggregation and alerting

### Tools Required
- Docker 20.10+
- Docker Compose 2.0+
- kubectl 1.25+ (for Kubernetes)
- Terraform 1.0+ (for IaC)

## Step 1: Environment Preparation

### 1.1 Create Production Environment File

```bash
# .env.production
DATABASE_URL=postgresql://user:$(python -c "import secrets; print(secrets.token_hex(32))")@db-host:5432/id_card_prod
RUST_LOG=info,id_card_system=debug
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# JWT Configuration
JWT_SECRET=$(openssl rand -hex 32)
JWT_EXPIRY=3600

# Stellar Network
STELLAR_NETWORK=public  # Use 'testnet' for testing
STELLAR_HORIZON_URL=https://horizon.stellar.org
STELLAR_NETWORK_PASSPHRASE=Public Global Stellar Network ; September 2015

# Stellar Account (create one at https://stellar.org)
STELLAR_ACCOUNT_ID=GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
STELLAR_SECRET_KEY=SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

# Features
ENABLE_BIOMETRIC_AUTH=true
ENABLE_STELLAR_PAYMENTS=true
ENABLE_QR_VERIFICATION=true
ENABLE_AUDIT_LOGGING=true

# CORS
CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://www.yourdomain.com

# Admin
ADMIN_EMAIL=admin@yourdomain.com
ADMIN_PASSWORD=$(python -c "import secrets; print(secrets.token_urlsafe(32))")

# Email (for notifications)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=noreply@yourdomain.com
SMTP_PASSWORD=$(python -c "import secrets; print(secrets.token_urlsafe(32))")
```

### 1.2 Generate Secure Secrets

```bash
#!/bin/bash

# Generate all secrets
JWT_SECRET=$(openssl rand -hex 32)
POSTGRES_PASSWORD=$(python -c "import secrets; print(secrets.token_urlsafe(32))")
REDIS_PASSWORD=$(python -c "import secrets; print(secrets.token_urlsafe(32))")
ADMIN_PASSWORD=$(python -c "import secrets; print(secrets.token_urlsafe(32))")

# Store securely (e.g., AWS Secrets Manager, HashiCorp Vault)
echo "Store these securely:"
echo "JWT_SECRET=$JWT_SECRET"
echo "POSTGRES_PASSWORD=$POSTGRES_PASSWORD"
echo "REDIS_PASSWORD=$REDIS_PASSWORD"
echo "ADMIN_PASSWORD=$ADMIN_PASSWORD"
```

## Step 2: Database Setup

### 2.1 PostgreSQL Database

```bash
# Using Docker
docker run -d \
  --name id-card-db \
  -e POSTGRES_USER=id_card_user \
  -e POSTGRES_PASSWORD=$POSTGRES_PASSWORD \
  -e POSTGRES_DB=id_card_prod \
  -v postgres_data:/var/lib/postgresql/data \
  -p 5432:5432 \
  postgres:15-alpine

# Or with managed service (AWS RDS, Azure Database, Google Cloud SQL)
# Follow provider's documentation
```

### 2.2 Run Migrations

```bash
cd backend

# Set database URL
export DATABASE_URL=postgresql://id_card_user:$POSTGRES_PASSWORD@localhost:5432/id_card_prod

# Run migrations
cargo sqlx migrate run
```

### 2.3 Backup Configuration

```bash
# Create backup script
cat > /usr/local/bin/backup-db.sh << 'EOF'
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/database"
mkdir -p $BACKUP_DIR

pg_dump $DATABASE_URL | gzip > $BACKUP_DIR/id_card_$DATE.sql.gz

# Keep only last 30 days
find $BACKUP_DIR -name "id_card_*.sql.gz" -mtime +30 -delete

# Upload to cloud storage
aws s3 cp $BACKUP_DIR/id_card_$DATE.sql.gz s3://backups/id-card/
EOF

chmod +x /usr/local/bin/backup-db.sh

# Schedule daily backups
(crontab -l 2>/dev/null; echo "0 2 * * * /usr/local/bin/backup-db.sh") | crontab -
```

## Step 3: Docker Deployment

### 3.1 Build Production Images

```bash
# Build backend
docker build \
  -f docker/backend.Dockerfile \
  --build-arg RUST_BACKTRACE=1 \
  -t id-card-backend:1.0.0 \
  -t id-card-backend:latest \
  .

# Build frontend
docker build \
  -f docker/react.Dockerfile \
  -t id-card-frontend:1.0.0 \
  -t id-card-frontend:latest \
  frontend_react/

# Push to registry
docker tag id-card-backend:latest registry.yourdomain.com/id-card-backend:latest
docker push registry.yourdomain.com/id-card-backend:latest

docker tag id-card-frontend:latest registry.yourdomain.com/id-card-frontend:latest
docker push registry.yourdomain.com/id-card-frontend:latest
```

### 3.2 Deploy with Docker Compose

```bash
# Production docker-compose
cat > docker-compose.prod.yml << 'EOF'
version: '3.8'

services:
  backend:
    image: registry.yourdomain.com/id-card-backend:latest
    container_name: id-card-backend
    restart: always
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=info
    volumes:
      - ./logs:/app/logs
    networks:
      - id-card-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  frontend:
    image: registry.yourdomain.com/id-card-frontend:latest
    container_name: id-card-frontend
    restart: always
    ports:
      - "80:80"
      - "443:443"
    environment:
      - VITE_API_URL=https://api.yourdomain.com/api/v1
    volumes:
      - /etc/letsencrypt:/etc/letsencrypt
      - ./nginx.conf:/etc/nginx/conf.d/default.conf
    networks:
      - id-card-network

networks:
  id-card-network:
    driver: bridge
EOF

# Deploy
docker-compose -f docker-compose.prod.yml up -d

# View logs
docker-compose -f docker-compose.prod.yml logs -f
```

## Step 4: Kubernetes Deployment

### 4.1 Create Kubernetes Manifests

```yaml
# k8s/backend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: id-card-backend
spec:
  replicas: 3
  selector:
    matchLabels:
      app: id-card-backend
  template:
    metadata:
      labels:
        app: id-card-backend
    spec:
      containers:
      - name: backend
        image: registry.yourdomain.com/id-card-backend:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: id-card-secrets
              key: database-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: id-card-secrets
              key: jwt-secret
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"

---
apiVersion: v1
kind: Service
metadata:
  name: id-card-backend-service
spec:
  selector:
    app: id-card-backend
  ports:
  - protocol: TCP
    port: 8080
    targetPort: 8080
  type: ClusterIP
```

### 4.2 Deploy to Kubernetes

```bash
# Create secrets
kubectl create secret generic id-card-secrets \
  --from-literal=database-url=$DATABASE_URL \
  --from-literal=jwt-secret=$JWT_SECRET

# Deploy
kubectl apply -f k8s/

# Verify deployment
kubectl get pods -l app=id-card-backend
kubectl logs -f deployment/id-card-backend
```

## Step 5: SSL/TLS Setup

### 5.1 Let's Encrypt Certificate

```bash
# Install Certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot certonly --nginx \
  -d yourdomain.com \
  -d api.yourdomain.com \
  --email admin@yourdomain.com

# Auto-renewal
sudo systemctl enable certbot.timer
sudo systemctl start certbot.timer
```

### 5.2 Nginx Configuration

```nginx
# /etc/nginx/sites-available/id-card

upstream backend {
    server localhost:8080;
}

# HTTP to HTTPS redirect
server {
    listen 80;
    server_name yourdomain.com api.yourdomain.com;
    return 301 https://$server_name$request_uri;
}

# HTTPS server
server {
    listen 443 ssl http2;
    server_name yourdomain.com api.yourdomain.com;

    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # API proxy
    location /api/ {
        proxy_pass http://backend/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Static files
    location / {
        root /var/www/html;
        try_files $uri $uri/ /index.html;
    }
}
```

## Step 6: Monitoring & Logging

### 6.1 Prometheus Metrics

```yaml
# docker-compose additions
prometheus:
  image: prom/prometheus:latest
  ports:
    - "9090:9090"
  volumes:
    - ./prometheus.yml:/etc/prometheus/prometheus.yml
    - prometheus_data:/prometheus
  command:
    - '--config.file=/etc/prometheus/prometheus.yml'

grafana:
  image: grafana/grafana:latest
  ports:
    - "3000:3000"
  environment:
    - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
  volumes:
    - grafana_data:/var/lib/grafana
```

### 6.2 Logging with ELK Stack

```yaml
elasticsearch:
  image: docker.elastic.co/elasticsearch/elasticsearch:8.0.0
  environment:
    - discovery.type=single-node
  volumes:
    - elasticsearch_data:/usr/share/elasticsearch/data

kibana:
  image: docker.elastic.co/kibana/kibana:8.0.0
  ports:
    - "5601:5601"
  environment:
    - ELASTICSEARCH_HOSTS=http://elasticsearch:9200

filebeat:
  image: docker.elastic.co/beats/filebeat:8.0.0
  volumes:
    - ./logs:/logs
    - /var/lib/docker/containers:/var/lib/docker/containers:ro
```

## Step 7: Security Hardening

### 7.1 Firewall Rules

```bash
# UFW
sudo ufw allow 22/tcp      # SSH
sudo ufw allow 80/tcp      # HTTP
sudo ufw allow 443/tcp     # HTTPS
sudo ufw enable
```

### 7.2 Security Checklist

- [ ] Update all packages
- [ ] Disable unnecessary services
- [ ] Configure fail2ban
- [ ] Enable SELinux/AppArmor
- [ ] Set up intrusion detection
- [ ] Configure log rotation
- [ ] Enable audit logging
- [ ] Restrict admin access
- [ ] Set up backup verification

## Step 8: Post-Deployment

### 8.1 Verification

```bash
# Test health endpoint
curl https://api.yourdomain.com/health

# Test registration
curl -X POST https://api.yourdomain.com/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{...}'

# Check logs
docker-compose logs -f backend
```

### 8.2 Performance Testing

```bash
# Load testing with Apache Bench
ab -n 1000 -c 10 https://api.yourdomain.com/health

# Or with locust
pip install locust
locust -f locustfile.py --host https://api.yourdomain.com
```

## Troubleshooting

### Deployment Issues

**Containers won't start**
```bash
# Check logs
docker-compose logs backend

# Verify environment variables
docker-compose config | grep JWT_SECRET

# Check database connection
docker exec id-card-backend psql -U id_card_user -c "SELECT 1"
```

**High memory usage**
```bash
# Monitor resources
docker stats

# Check for memory leaks
valgrind backend --leak-check=full
```

**Database connection errors**
```bash
# Test connection
psql postgresql://user:pass@host:5432/db

# Check firewall
telnet db-host 5432
```

## Maintenance

### Regular Tasks

- **Daily**: Review logs, backup database
- **Weekly**: Update dependencies, security patches
- **Monthly**: Full system backup, disaster recovery test
- **Quarterly**: Security audit, performance review
- **Annually**: Full system penetration test

### Upgrade Procedure

```bash
# 1. Backup everything
./backup-db.sh

# 2. Pull new images
docker pull registry.yourdomain.com/id-card-backend:latest

# 3. Update docker-compose
git pull origin main

# 4. Restart services
docker-compose -f docker-compose.prod.yml up -d

# 5. Run migrations if needed
docker-compose exec backend migrate

# 6. Verify
curl https://api.yourdomain.com/health
```

## Support

- **Issues**: Check logs first
- **Documentation**: See docs/ directory
- **Team**: Contact your DevOps team
- **Escalation**: Contact engineering lead

---

For additional help, see [docs/SECURITY.md](SECURITY.md) and [docs/ARCHITECTURE.md](ARCHITECTURE.md)
