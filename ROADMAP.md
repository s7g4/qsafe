# Development Roadmap: Q-Safe Gateway

This document maps out the specific development milestones required to rewrite the Q-Safe gateway from a student prototype to a production-grade, hardware-assisted secure messaging service.

## Milestone 1: Database Migration & Configuration Refactoring
* **Goal**: Implement standard environment parsing and a database schema migration lifecycle.
* **Why**: Executing schema queries at boot blocks clustering and is untrackable. Hardcoded configurations lead to silent launch failures.
* **Inputs**: Legacy SQL queries inside `host-server/src/database.rs` and raw environment variables in `host-server/src/main.rs`.
* **Outputs**:
  - Validated config schema in `host-server/src/config.rs` using `envy` and `serde`.
  - Database schema scripts in `host-server/migrations/`.
- **Dependencies**: Workspace structure set up.
- **Risks**: Local database version mismatches during migration runs.
- **Success Criteria**: The server validates all configuration keys at launch and applies database schemas via `sqlx migrate run`.
- **Metrics**: 0 raw schema SQL strings in code; 100% of config variables validated at boot.
- **Expected Commits**:
  - `refactor: introduce validated configuration loader`
  - `feat: migrate database table setup to SQLx migration files`

## Milestone 2: Secure Authentication & Error Handling Framework
* **Goal**: Introduce Argon2id password hashing, a dual-token authentication system, and a panic-free error propagation system.
* **Why**: Basic JWTs force constant relogins. Default bcrypt is slow. Unhandled `unwrap()` calls crash server threads in production.
* **Inputs**: JWT generation in `host-server/src/auth.rs` and Axum handlers in `host-server/src/main.rs`.
* **Outputs**:
  - Access Token (JWT) and HttpOnly Cookie-based Refresh Token rotation flow.
  - Custom `QSafeError` enum using `thiserror` mapping to HTTP status codes.
- **Dependencies**: Milestone 1 config setup.
- **Risks**: JWT validation clock skew under high loads.
- **Success Criteria**: The server returns clean JSON error messages and correct HTTP status codes instead of dropping connections or panicking.
- **Metrics**: 0 instances of `unwrap()` or `expect()` in service modules.
- **Expected Commits**:
  - `feat: implement Argon2id hashing and cookie-based token rotation`
  - `refactor: introduce unified QSafeError to eliminate system panics`

## Milestone 3: Hardware Interface Driver & Local Simulation
* **Goal**: Design the TLV serial communication driver and the host-side hardware-in-the-loop (HIL) software simulator.
* **Why**: The server needs to exchange keys with the HSM over USB, but we must be able to run and test the codebase locally without physical hardware.
* **Inputs**: Hardware specs defined in `RESEARCH.md` and `common` crate structure.
* **Outputs**:
  - USB Serial Driver in `host-server/src/hardware.rs` using `serialport-rs`.
  - In-memory mock serial responder in `host-server/src/hardware/mock.rs` that simulates RP2040 key operations.
- **Dependencies**: Milestone 2 error handling.
- **Risks**: Serial data buffer alignment errors during byte transfers.
- **Success Criteria**: Host driver successfully establishes connections, runs CRC-16 checks on packet frames, and handles simulated timeouts.
- **Metrics**: 100% of packet transmissions verified via CRC-16 checks.
- **Expected Commits**:
  - `feat: implement TLV packet framing and CRC-16 check logic`
  - `test: build host-side hardware-in-the-loop serial simulator`

## Milestone 4: Hybrid Crypto Standards & Memory Zeroization
* **Goal**: Refactor cryptographic wrappers to use standardized constructs and protect key parameters in memory.
* **Why**: Custom cryptographic implementations can leak keys. Sensitive key material must be immediately cleared from RAM.
* **Inputs**: Hybrid agreement routines in `host-server/src/crypto.rs`.
* **Outputs**:
  - Unified crypto gateway in `host-server/src/crypto.rs` using standardized HPKE primitives.
  - Memory zeroization traits (`zeroize`) integrated into key structures.
- **Dependencies**: Milestone 3 serial driver.
- **Risks**: Key decapsulation computation latency.
- **Success Criteria**: Cryptographic keys are securely derived and zeroized immediately on drop.
- **Metrics**: 100% of ephemeral key structures zeroized on drop.
- **Expected Commits**:
  - `refactor: upgrade cryptographic primitives to HPKE standards`
  - `security: integrate memory zeroization on all key drops`

## Milestone 5: Async WebSocket Registry & Concurrency Refactoring
* **Goal**: Rewrite the connection loops to resolve the client registry ownership challenges and route messages between connected clients.
* **Why**: WebSockets must route messages concurrently to distinct target sockets without blocking threads or deadlocking.
* **Inputs**: WebSocket event loops in `host-server/src/websocket.rs`.
* **Outputs**:
  - Thread-safe actor connection manager using Tokio channel loops.
  - PostgreSQL message buffering database tables for offline queues.
- **Dependencies**: Milestone 4 crypto gateway.
- **Risks**: Lock contention in the client mapping registry under high message volumes.
- **Success Criteria**: Multiple distinct WebSocket clients can connect, authenticate, and exchange encrypted messages.
- **Metrics**: Message dispatch latency under 5ms.
- **Expected Commits**:
  - `feat: build async channel-based WebSocket client registry`
  - `feat: implement database-buffered message delivery queues`

## Milestone 6: Observability, Tracing, & Telemetry
* **Goal**: Instrument the server with structured logging and metric collection endpoints.
* **Why**: Standard prints do not correlate requests, trace latencies, or check downstream services.
* **Inputs**: Entire workspace codebase.
- **Outputs**:
  - Logger initialization in `host-server/src/main.rs` using `tracing`.
  - Prometheus metric compilation routes (`/metrics`).
- **Dependencies**: Milestone 5 concurrency model.
- **Risks**: Accidental logging of user payloads or keys.
- **Success Criteria**: Gateway records JSON logs with unique Request IDs and exposes resource performance metrics.
- **Metrics**: 100% of HTTP API requests record processing latencies.
- **Expected Commits**:
  - `feat: integrate tracing logger and request correlation IDs`
  - `feat: expose Prometheus metrics endpoint`
