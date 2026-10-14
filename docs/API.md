# API Reference

## Authentication

All API endpoints (except `/auth/register` and `/auth/login`) require authentication via JWT token.

### Headers
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

## Endpoints

### Authentication

#### Register User
```
POST /api/v1/auth/register
```

Request:
```json
{
  "email": "student@university.edu",
  "password": "SecurePassword123!",
  "name": "John Doe",
  "institution": "Federal University of Technology Minna"
}
```

Response:
```json
{
  "status": "success",
  "user_id": "uuid",
  "email": "student@university.edu"
}
```

#### Login
```
POST /api/v1/auth/login
```

Request:
```json
{
  "email": "student@university.edu",
  "password": "SecurePassword123!"
}
```

Response:
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "expires_in": 3600
}
```

### Users

#### Get Current User
```
GET /api/v1/users/me
```

Response:
```json
{
  "id": "uuid",
  "email": "student@university.edu",
  "name": "John Doe"
}
```

### Credentials

#### List Credentials
```
GET /api/v1/credentials
```

Response:
```json
{
  "credentials": [
    {
      "id": "uuid",
      "status": "active",
      "issued_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1
}
```

#### Get Credential
```
GET /api/v1/credentials/{credential_id}
```

### Blockchain

#### Get Blockchain
```
GET /api/v1/blockchain/chain
```

Response:
```json
{
  "blocks": [...],
  "length": 1
}
```

## Error Responses

### 400 Bad Request
```json
{
  "status": "error",
  "code": "VALIDATION_ERROR",
  "message": "Invalid request parameters"
}
```

### 401 Unauthorized
```json
{
  "status": "error",
  "code": "UNAUTHORIZED",
  "message": "Authentication required"
}
```

### 404 Not Found
```json
{
  "status": "error",
  "code": "NOT_FOUND",
  "message": "Resource not found"
}
```

## Rate Limiting

- **Default Limit**: 100 requests per minute
- **Headers**:
  - `X-RateLimit-Limit`
  - `X-RateLimit-Remaining`
  - `X-RateLimit-Reset`
