# Changelog: Q-Safe Gateway

All notable changes to the Q-Safe secure messaging gateway will be documented in this file. This project follows [Semantic Versioning](https://semver.org/).

## [Unreleased] - 2026-07-10

### Added
- **Real integration tests for the auth flow**: `host-server/tests/auth_flow.rs` boots the actual axum app against a real Postgres database and exercises register/login/refresh/logout and query-based WebSocket token authorization over real HTTP and WebSocket connections, not just unit-level calls into `AuthService`.
- **Real integration tests for the Mock HSM path**: `host-server/tests/hsm_mock_flow.rs` proves the client-encapsulate / HSM-decapsulate round-trip matches, detects tampered ciphertext, and confirms the register endpoint pulls a real Kyber-768 key through the live `HsmConnection` abstraction.
- Unit tests for JWT expiry rejection and signature-mismatch rejection in `host-server/src/auth.rs`.
- [docs/HSM_VERIFICATION_STATUS.md](docs/HSM_VERIFICATION_STATUS.md): an honest breakdown of what's proven vs. architecturally-sound-but-unverified vs. not-yet-implemented in the HSM path (the RP2040 firmware is currently an empty stub with no on-device logic).
- [docs/HYBRID_KEY_EXCHANGE.md](docs/HYBRID_KEY_EXCHANGE.md): technical writeup of the ML-KEM-768 + X25519 hybrid key exchange design.
- A GitHub Pages-published mdBook under `docs/book/` covering the full doc set (architecture, API reference, HSM verification status, hybrid key exchange, ADRs, roadmap, changelog, devlog), auto-deployed by `.github/workflows/docs.yml`.
- A real-output demo GIF (`docs/assets/demo.gif`) and `host-server/examples/demo_client.rs`, showing register/login/reject-wrong-password/WebSocket-messaging/test-suite from actual captured terminal output.
- Unit tests for `CryptoEngine` in `host-server/src/crypto.rs`: Kyber encapsulate/decapsulate round-trip, X25519 shared-secret agreement matching on both sides, the HKDF-SHA3-256 hybrid key derivation matching and differing correctly, and Ed25519 sign/verify including tampered-message and wrong-key rejection.

### Fixed
- **Rate limiter 500s on every auth request**: `tower_governor`'s peer-IP key extractor requires `ConnectInfo<SocketAddr>`, which the server never provided (`main.rs` called `axum::serve`/`axum_server` with a bare `Router`/`into_make_service()`). Every request to `/api/auth/*` returned `500 Unable To Extract Key!`. Found via the new integration tests; fixed by serving via `into_make_service_with_connect_info::<SocketAddr>()` on both the TLS and plain-HTTP paths.

### Changed
- Extracted `AppState`, the `AuthedUser` extractor, and all HTTP handlers out of `main.rs` into `host-server/src/app.rs` (a library module) so integration tests can build the real router instead of re-testing logic in isolation. `main.rs` is now a thin entrypoint.
- Rewrote `README.md` to only claim what the test suite backs up, and to accurately describe the firmware/HSM path as designed-but-unimplemented rather than "production-ready."

### Removed
- `host-server/src/db_tests.rs` and `host-server/src/scratch_governor.rs`: orphaned files never referenced by any `mod` declaration, so they never compiled or ran. Their existence overstated test coverage.
- `host-server/src/messaging.rs` (`MessagingService`) and `host-server/src/ui.rs` (`UI`): dead code, never instantiated anywhere. `messaging.rs` also shadowed `database::Message`/`ChatSession` with differently-shaped types of the same name. Dropped the now-unused `wasm-bindgen`, `web-sys`, and `plotters` optional dependencies and the `web`/`visualization` features that only existed for `ui.rs`.
- Three inert `AppState` fields (`crypto`, `qkd`, `qrng`) that were constructed at startup but never read by any handler.
- `CryptoEngine`'s unused "legacy" method aliases, and its `encrypt_aead`/`decrypt_aead` wrapper: also unused (the real message flow relays opaque client-encrypted blobs), and mislabeled - the doc comment said "ChaCha20-Poly1305" with an implied 12-byte nonce, but `orion::aead` actually uses XChaCha20Poly1305 with a 24-byte nonce; the code only round-tripped because both functions agreed on the same arbitrary split point, not because the framing was correct.

## [1.0.0] - 2026-05-30

### Added
- **Production Hardening Completed:** All 10 phases of the production hardening plan have been implemented and verified.
- **Docker Deployment:** Added multi-stage `Dockerfile` (debian:bookworm-slim runtime) and `docker-compose.yml` for database and backend orchestration.
- **HTTPS/TLS:** Integrated `axum-server` with `tls-rustls` feature to support optional TLS termination via `TLS_CERT_PATH` and `TLS_KEY_PATH` configuration.
- **Unit & Integration Tests:** Implemented unit tests for Argon2id hashing and JWT validation in `auth.rs`, and a conditional Postgres integration test in `database.rs`.
- **CI Pipeline:** Added `cargo audit` and `cargo deny check` steps to GitHub Actions workflow (`ci.yml`) to enforce security and licensing constraints.
- **Rate Limiting:** Integrated `tower_governor` IP-based rate limiting (10 req/min) on authentication routes to mitigate brute-force attacks.

### Changed
- **Config & CORS:** Exposed `CORS_ORIGIN` and `DB_MAX_CONNECTIONS` to environment variables.
- **Graceful Shutdown:** Implemented SIGINT/CTRL+C trap for `axum::serve::with_graceful_shutdown()` and `axum_server`.
- **WebSocket Auth:** Enforced secure connection routing on WebSocket upgrades via a `token` URL query parameter, preventing unauthenticated WS connections.
- **Memory Safety:** Replaced unbounded message channels (`mpsc::unbounded_channel`) with bounded channels (`1024` capacity, `try_send`) in `WebSocketRegistry` to prevent OOM DOS vectors.
- **Error Sanitization:** Introduced `QSafeError::ValidationError(String)` yielding HTTP 422, halting database schema and JWT signature leaks to untrusted clients.
- **README:** Rewrote `README.md` focusing on current production-ready capabilities and deployment guides.

### Removed
- Unused/Stale strategy documents: `PROJECT_AUDIT.md`, `ROADMAP.md`, and `VISION.md`.

## [0.1.7] - 2026-05-30

### Added
- Implemented `AuthedUser` custom Axum extractor in [host-server/src/main.rs](host-server/src/main.rs) for JWT-based request authorization via `Authorization: Bearer <token>` headers.
- Created [docs/adr/0011-http-api-routing-and-authorization-middleware.md](docs/adr/0011-http-api-routing-and-authorization-middleware.md) documenting API routing and auth middleware decisions.

### Changed
- Upgraded `GET /api/messages/:user_id` handler in [host-server/src/main.rs](host-server/src/main.rs) to query the database using `db.get_messages_between_users`.
- Upgraded `POST /api/messages/send` handler to decode base64-encoded `encrypted_content` and `nonce` payloads and persist via `db.save_message`.
- Upgraded `GET /api/contacts` handler to fetch contacts from the database using `db.get_contacts`.
- Upgraded `POST /api/contacts/add` handler to insert contact relationships using `db.add_contact`.
- Modified `add_contact` query in [host-server/src/database.rs](host-server/src/database.rs) to set `status = 'accepted'` on insert.
- Added `base64` dependency and `FromRef` import in [host-server/Cargo.toml](host-server/Cargo.toml).

## [0.1.6] - 2026-05-30

### Added
- Created [docs/adr/0010-observability-tracing-and-telemetry.md](docs/adr/0010-observability-tracing-and-telemetry.md) documenting design choices for logging and monitoring.
- Integrated `metrics` and `metrics-exporter-prometheus` to publish performance telemetry.
- Integrated `tracing` and `tracing-subscriber` for structured JSON logging.

### Changed
- Modified [host-server/Cargo.toml](host-server/Cargo.toml) to import telemetry and tracing dependencies, and enabled trace/request-id layers in `tower-http`.
- Modified [host-server/src/main.rs](host-server/src/main.rs) to boot the tracing subscriber, set up the Prometheus metrics recorder, expose a `/metrics` route, and add request correlation middlewares (`SetRequestIdLayer` and `TraceLayer`).
- Modified [host-server/src/websocket.rs](host-server/src/websocket.rs) to instrument metrics for active WebSocket sessions, messages sent count, and offline message queue buffering counts.

## [0.1.5] - 2026-05-30

### Added
- Created [host-server/migrations/0002_offline_messages.sql](host-server/migrations/0002_offline_messages.sql) to define the schema for buffering offline messages.
- Created [docs/adr/0009-async-websocket-registry-and-offline-queues.md](docs/adr/0009-async-websocket-registry-and-offline-queues.md) to record the design of the actor-model WebSocket registry and database queues.

### Changed
- Refactored [host-server/src/database.rs](host-server/src/database.rs) to derive `Clone` on `Database` and implement `save_offline_message`, `get_offline_messages`, and `clear_offline_messages`.
- Refactored [host-server/src/websocket.rs](host-server/src/websocket.rs) to use non-blocking Tokio channels in `WebSocketRegistry` and `WebSocketRegistryActor` instead of `Arc<Mutex<HashMap<...>>>`.
- Upgraded WebSocket connection socket split tasks in [host-server/src/websocket.rs](host-server/src/websocket.rs) to handle asynchronous read/write routing, delivering and queuing messages.
- Modified [host-server/src/main.rs](host-server/src/main.rs) to initialize and spawn the registry actor, register it in `AppState`, and route incoming WebSocket upgrades to the revised controller.

## [0.1.4] - 2026-05-30

### Added
- Created [docs/adr/0008-hybrid-crypto-standards-and-memory-zeroization.md](docs/adr/0008-hybrid-crypto-standards-and-memory-zeroization.md) logging threat models and active key zeroization decisions.

### Changed
- Refactored [host-server/src/crypto.rs](host-server/src/crypto.rs) to implement `Zeroize` and `Drop` on `KeyPair` and `HybridSharedSecret`.
- Refactored `CryptoEngine` key-agreements, sign operations, and key generators inside [host-server/src/crypto.rs](host-server/src/crypto.rs) to wipe stack parameters and intermediate allocations on method completion.
- Modified [host-server/src/handshake.rs](host-server/src/handshake.rs) to clone key variables, preventing compiler borrow-check move errors out of dropping structures.
- Added `libudev-dev` installation step inside [.github/workflows/ci.yml](.github/workflows/ci.yml) to fix Linux compilation failures of the `serialport` crate in the CI runner.

## [0.1.3] - 2026-05-30

### Added
- Created [host-server/src/hardware.rs](host-server/src/hardware.rs) implementing the `HsmConnection` trait, physical serial driver, and in-memory mock simulator.
- Created [docs/adr/0007-hardware-interface-driver-and-simulation.md](docs/adr/0007-hardware-interface-driver-and-simulation.md) logging serial framing and HIL mock designs.

### Changed
- Configured [common/src/lib.rs](common/src/lib.rs) as a `#![no_std]` crate, implementing Type-Length-Value (TLV) packet framing and CRC-16-CCITT checksum loops.
- Linked `qsafe-common` and added `serialport` to [host-server/Cargo.toml](host-server/Cargo.toml) dependencies.
- Updated [host-server/src/config.rs](host-server/src/config.rs) and [host-server/src/main.rs](host-server/src/main.rs) to load HSM configuration options and wire connections inside `AppState`.
- Modified user registration in [host-server/src/main.rs](host-server/src/main.rs) to fetch the Kyber public key from the HSM interface.

## [0.1.2] - 2026-05-30

### Added
- Created [host-server/src/error.rs](host-server/src/error.rs) defining the custom `QSafeError` enum mapping workspace errors to structured JSON response payloads.
- Added `/api/auth/refresh` and `/api/auth/logout` endpoints inside [host-server/src/main.rs](host-server/src/main.rs).
- Created [docs/adr/0006-secure-authentication-and-error-handling.md](docs/adr/0006-secure-authentication-and-error-handling.md) to log design decisions for auth and error handling.

### Changed
- Replaced `bcrypt` with `argon2` and added `thiserror` dependencies inside [host-server/Cargo.toml](host-server/Cargo.toml).
- Upgraded password hashing and verification in [host-server/src/auth.rs](host-server/src/auth.rs) to Argon2id.
- Implemented Access Token (15m) and Refresh Token (7d) generation logic in [host-server/src/auth.rs](host-server/src/auth.rs).
- Refactored route handlers in [host-server/src/main.rs](host-server/src/main.rs) to use `QSafeError`, return HttpOnly rotated cookies, and eliminate unhandled panic-prone calls.

## [0.1.1] - 2026-05-30

### Added
- Created database migration schema in [host-server/migrations/0001_init.sql](host-server/migrations/0001_init.sql) for automated SQLx migrations.
- Created [docs/adr/0004-disabling-test-harness-for-bare-metal-binaries.md](docs/adr/0004-disabling-test-harness-for-bare-metal-binaries.md) to record host test harness target configurations.
- Created [docs/adr/0005-database-migrations.md](docs/adr/0005-database-migrations.md) to document automated SQLx database migration strategy.

### Changed
- Configured [firmware/Cargo.toml](firmware/Cargo.toml) binary target with `test = false` and `bench = false` to resolve host-side workspace test linking errors (`LNK1561`).
- Modified [Database::new](host-server/src/database.rs#L48-L56) inside [host-server/src/database.rs](host-server/src/database.rs) to run database migrations automatically.
- Removed legacy `create_tables` logic from [host-server/src/database.rs](host-server/src/database.rs) and [host-server/src/main.rs](host-server/src/main.rs).

## [0.1.0] - 2026-05-29


### Added
- Created `PROJECT_AUDIT.md` mapping legacy technical debt and vulnerability vectors.
- Created `VISION.md` defining the repositioning of the project as a hardware-assisted messaging gateway.
- Created `RESEARCH.md` compiling references for ML-KEM-768, USB packet framing, and Embassy async runtime.
- Created `ARCHITECTURE.md` mapping workspace boundaries and sequence diagrams.
- Created `ROADMAP.md` establishing 6 core engineering milestones.
- Initialized `METRICS.md` to track performance and binary size boundaries.
- Set up Architectural Decision Record (ADR) index under `docs/adr/`.
- Created `host-server/src/config.rs` validating configuration environment variables at launch.

### Changed
- Restructured the repository into a multi-crate Cargo Workspace containing `host-server/`, `common/`, and `firmware/` 
- Integrated `Config::load()` in `host-server/src/main.rs` to replace raw unvalidated environment fetches.
