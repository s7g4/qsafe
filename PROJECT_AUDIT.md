# Project Audit: Q-Safe

This audit evaluates the Q-Safe secure messaging system to identify implementation gaps, architectural flaws, technical debt, and security vulnerabilities. This document defines the engineering baseline for refactoring the codebase.

---

## 1. Context & Goals
* **Original Project Goal**: To implement a secure messaging backend in Rust featuring post-quantum cryptography (Kyber) and simulated quantum communication protocols (QKD BB84/E91, QRNG) over HTTP APIs and WebSockets.
* **The Refactoring Narrative**: I built the initial part of this project when I was a student to learn basic Rust API concepts and cryptography. Now, as I strive for a professional role in systems and embedded engineering, I am using this project as a research playground. My goal is to refactor it from a student prototype to a production-ready Cargo Workspace integrating bare-metal Embedded Rust with secure messaging mechanics.

---

## 2. Current State Analysis

### What Exists
- **HTTP Routing & API**: A basic Axum HTTP web server with routes for registration, login, messages, and contacts ([src/main.rs](src/main.rs)).
- **Core Cryptographic Structures**: Raw wrappers for Kyber-768, X25519, and Ed25519 signatures ([src/crypto.rs](src/crypto.rs)).
- **Postgres Integration**: Hardcoded table setups and basic CRUD queries using SQLx ([src/database.rs](src/database.rs)).
- **QKD & QRNG Simulators**: BB84/E91 sifting channels and random state simulation ([src/qkd.rs](src/qkd.rs), [src/qrng.rs](src/qrng.rs)).
- **Authentication**: JWT token generation and bcrypt password hashing ([src/auth.rs](src/auth.rs)).

### What is Broken
- **WebSocket State Tracking**: The socket handler in [src/websocket.rs](src/websocket.rs) splits streams but skips storing active connections in the shared state map to bypass compiler ownership checks. It acts purely as a local echo stub.
- **Stub API Route Implementations**: Message sending, message list fetching, and contact adding APIs return empty dummy JSON responses without executing database queries or routing messages.
- **Inline Table Init**: Database tables are created via inline raw SQL query strings at runtime during startup, which blocks proper schema management and migrations.

### What Should Be Removed
- **Inline SQL Schemas**: The tables creation query inside `Database::create_tables` should be completely removed in favor of structured database migration files.
- **Mock Handshake Stubs inside Cryptography**: Simulated network state check stubs inside [src/handshake.rs](src/handshake.rs) must be removed to separate networking logic from mathematical primitives.

### What Should Be Rewritten
- **Environment & Configuration Engine**: Variable parsing using raw `std::env::var` inside [src/main.rs](src/main.rs) must be rewritten using a configuration load crate (e.g., `config` or `envy`) with boot validation checks.
- **WebSocket Connection Registry**: Rebuild the client registry with a thread-safe actor-like or connection mapping pattern (using channels) to safely route messages between connections.
- **Unified Error Handling**: Replace all system-panicking `unwrap()` and `expect()` calls with custom error enums using `thiserror`.

### What Should Be Preserved
- **Hybrid Security Philosophy**: The design of combining classical cryptography (X25519) with post-quantum algorithms (Kyber) is excellent and remains a core system feature.
- **Simulators as Test Assets**: Keep the QKD and QRNG simulation frameworks isolated as diagnostic components for test suites.

---

## 3. Vulnerabilities & Architectural Debt

- **Key Security in Host Memory**: Private keys are handled as standard byte vectors in RAM without memory protections, making them vulnerable to memory scraping.
- **Monolithic State & Scale Blockers**: Shared service state uses global locks, and in-memory connection tables block horizontal backend scaling.
- **Observability Deficiencies**: Logging is limited to raw stdout `println!` calls without structure, levels, or correlation IDs.
- **Testing Gaps**: The project has zero unit or integration tests.
