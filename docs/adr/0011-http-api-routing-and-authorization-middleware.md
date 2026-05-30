# 11. HTTP API Routing and Authorization Middleware

## Status

Proposed

## Context

In the legacy gateway prototype, the HTTP endpoints for messages and contacts served as simple stubs returning hardcoded mock JSON objects. They did not interface with the database or verify request authenticity.

To bridge the gap to a fully production-ready gateway, we must:
1. Secure the messaging and contact routes by verifying the client's JWT access token on every request.
2. Implement backend database CRUD logic for these endpoints, allowing actual message exchange and contact tracking.

## Decision

We will:
1. **Custom Extractors for JWT Authentication**:
   - Implement a custom `AuthedUser` struct in [main.rs](../../host-server/src/main.rs) that implements Axum's `FromRequestParts` trait.
   - The extractor will extract the `Authorization` header, verify the bearer token using `AuthService`, and return an authorized session identity context.
2. **Database-Backed Router API Endpoints**:
   - Refactor message and contact handlers in [main.rs](../../host-server/src/main.rs) to query [database.rs](../../host-server/src/database.rs).
   - Use Base64 decoding for message payloads (`encrypted_content` and `nonce`) during serialization boundary transitions.
   - Update `add_contact` in [database.rs](../../host-server/src/database.rs) to store relationships with an immediate `'accepted'` status so they can be retrieved by contacts queries immediately.

## Consequences

- **Security**: Messaging and contact routes are now fully protected by JWT verification. Unauthenticated requests are rejected on extractor boundary entry.
- **Functionality**: Replaced all hardcoded stubs with database operations.
- **Performance**: Axum's async request-parts extraction is fast and avoids cloning overhead.
