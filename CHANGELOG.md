# Changelog: Q-Safe Gateway

All notable changes to the Q-Safe secure messaging gateway will be documented in this file. This project follows [Semantic Versioning](https://semver.org/).

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
