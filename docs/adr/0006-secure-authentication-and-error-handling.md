# ADR 0006: Argon2id Hashing, Dual-Token Auth, and Structured Error Handling

## Status
Accepted

## Context
1. **Password Hashing Security**: The legacy system used `bcrypt` with default parameters. Modern security standards (e.g. OWASP, NIST) recommend Argon2id as the state-of-the-art password hashing algorithm due to its configurable memory, time, and parallelization parameters which resist GPU-based side-channel attacks.
2. **Session Expiration & Usability**: Standard JWTs force users to re-login frequently. To prevent this without compromising security, we need a dual-token system: a short-lived Access Token in memory, and a long-lived Refresh Token in a secure, `HttpOnly`, `Secure`, `SameSite=Strict` cookie.
3. **Resiliency & Observability**: Raw `unwrap()` and `expect()` calls in request handlers or database logic can crash the thread/process or return raw connection closures to the client. A type-safe error propagation model is needed to return consistent JSON error envelopes.

## Decision
1. **Password Hashing**: Replace `bcrypt` with the `argon2` crate using the standard Argon2id algorithm.
2. **Dual-Token System**:
   - Register/Login return an Access Token (valid for 15 minutes) and set a secure `HttpOnly` refresh cookie (valid for 7 days).
   - Implement `/api/auth/refresh` for rotating tokens.
   - Implement `/api/auth/logout` for deleting refresh cookies.
3. **Error System**: Define a custom `QSafeError` enum using `thiserror` that implements `axum::response::IntoResponse` to return:
   ```json
   {
     "success": false,
     "data": null,
     "message": "<Error Description>"
   }
   ```
   with appropriate HTTP status codes (400, 401, 403, 404, 409, 500).

## Consequences

### Positive
- **Side-Channel Resistant Hashing**: Upgraded security against brute-force and GPU hashing attacks.
- **Cookie Security**: Refresh tokens are isolated from browser scripts (`HttpOnly`), preventing Cross-Site Scripting (XSS) extraction.
- **Robustness**: 100% of routes return type-safe, structured error payloads instead of panicking.

### Negative
- Client must handle both JSON response processing (for the Access Token) and browser cookie handling (automatic for standard web environments).
