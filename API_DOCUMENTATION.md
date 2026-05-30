# Q-Safe API Reference

Q-Safe operates as a headless backend API infrastructure. This document outlines the REST endpoints for authentication and contact management, as well as the real-time WebSocket protocol for encrypted messaging.

## 1. Authentication (`/api/auth`)

All authentication endpoints are rate-limited to 10 requests per minute per IP address. Q-Safe uses a dual-token architecture: a short-lived Access Token (JWT) returned in the JSON response, and a long-lived Refresh Token stored securely in an `HttpOnly` cookie.

### 1.1 Register
Registers a new user and returns an access token.
- **Endpoint**: `POST /api/auth/register`
- **Body**:
  ```json
  {
    "username": "alice",
    "email": "alice@example.com",
    "password": "supersecretpassword"
  }
  ```
- **Response** (200 OK):
  ```json
  {
    "success": true,
    "message": "User registered successfully",
    "data": {
      "access_token": "eyJhbG...",
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "alice"
    }
  }
  ```
  *(Also sets `refresh_token` HttpOnly cookie)*

### 1.2 Login
Authenticates an existing user.
- **Endpoint**: `POST /api/auth/login`
- **Body**:
  ```json
  {
    "username": "alice",
    "password": "supersecretpassword"
  }
  ```
- **Response** (200 OK): Same as Register.

### 1.3 Refresh Token
Generates a new access token using the HttpOnly refresh cookie.
- **Endpoint**: `POST /api/auth/refresh`
- **Headers**: Must include cookies.
- **Response** (200 OK):
  ```json
  {
    "success": true,
    "message": "Token refreshed",
    "data": {
      "access_token": "eyJhbG..."
    }
  }
  ```

### 1.4 Logout
Clears the refresh token cookie.
- **Endpoint**: `POST /api/auth/logout`
- **Response** (200 OK): `{ "success": true, "message": "Logged out" }`

## 2. Contacts (`/api/contacts`)

Requires the `Authorization: Bearer <access_token>` header.

### 2.1 Get Contacts
Retrieves the list of connected contacts for the authenticated user.
- **Endpoint**: `GET /api/contacts`
- **Response** (200 OK):
  ```json
  {
    "success": true,
    "message": "Contacts fetched",
    "data": [
      {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "username": "bob",
        "status": "accepted"
      }
    ]
  }
  ```

### 2.2 Add Contact
Initiates a connection with a target user.
- **Endpoint**: `POST /api/contacts/add`
- **Body**:
  ```json
  {
    "target_username": "bob"
  }
  ```
- **Response** (200 OK): `{ "success": true, "message": "Contact added" }`

## 3. Real-Time Messaging (`/ws`)

Q-Safe utilizes WebSockets for instantaneous, low-latency message delivery. All payloads are expected to be encrypted prior to transit using the hardware-derived keys.

### 3.1 Connection
The WebSocket connection must be authenticated via a URL query parameter containing the JWT access token.
- **URL**: `ws://<host>:3000/ws?token=<access_token>`

### 3.2 Sending a Message (Client to Server)
To route a message to a contact, send a JSON string through the open socket:
```json
{
  "type": "chat",
  "recipient_id": "123e4567-e89b-12d3-a456-426614174000",
  "content": "<base64_encrypted_payload>",
  "nonce": "<base64_nonce>"
}
```

### 3.3 Receiving a Message (Server to Client)
When a contact sends a message to you, the server pushes the following JSON payload:
```json
{
  "sender_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "<base64_encrypted_payload>",
  "nonce": "<base64_nonce>",
  "timestamp": "2026-05-30T16:00:00Z"
}
```

## 4. Fallback Messaging (`/api/messages`)

If WebSockets are disconnected or firewalled, the REST API can be used to poll and send messages.

### 4.1 Get Message History
- **Endpoint**: `GET /api/messages/:user_id`
- **Response** (200 OK): Returns an array of historical message objects between the caller and the specified `user_id`.

### 4.2 Send Message (REST Fallback)
- **Endpoint**: `POST /api/messages/send`
- **Body**:
  ```json
  {
    "recipient_id": "123e4567-...",
    "encrypted_content": "<base64>",
    "nonce": "<base64>"
  }
  ```
- **Response** (200 OK): `{ "success": true, "message": "Message sent" }`
