# Changelog: Q-Safe Gateway

All notable changes to the Q-Safe secure messaging gateway will be documented in this file. This project follows [Semantic Versioning](https://semver.org/).

## [0.1.2] - 2026-05-30

### Added
- Created [host-server/src/error.rs](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/error.rs) defining the custom `QSafeError` enum mapping workspace errors to structured JSON response payloads.
- Added `/api/auth/refresh` and `/api/auth/logout` endpoints inside [host-server/src/main.rs](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/main.rs).
- Created [docs/adr/0006-secure-authentication-and-error-handling.md](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/docs/adr/0006-secure-authentication-and-error-handling.md) to log design decisions for auth and error handling.

### Changed
- Replaced `bcrypt` with `argon2` and added `thiserror` dependencies inside [host-server/Cargo.toml](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/Cargo.toml).
- Upgraded password hashing and verification in [host-server/src/auth.rs](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/auth.rs) to Argon2id.
- Implemented Access Token (15m) and Refresh Token (7d) generation logic in [host-server/src/auth.rs](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/auth.rs).
- Refactored route handlers in [host-server/src/main.rs](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/main.rs) to use `QSafeError`, return HttpOnly rotated cookies, and eliminate unhandled panic-prone calls.

## [0.1.1] - 2026-05-30

### Added
- Created database migration schema in [host-server/migrations/0001_init.sql](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/migrations/0001_init.sql) for automated SQLx migrations.
- Created [docs/adr/0004-disabling-test-harness-for-bare-metal-binaries.md](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/docs/adr/0004-disabling-test-harness-for-bare-metal-binaries.md) to record host test harness target configurations.
- Created [docs/adr/0005-database-migrations.md](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/docs/adr/0005-database-migrations.md) to document automated SQLx database migration strategy.

### Changed
- Configured [firmware/Cargo.toml](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/firmware/Cargo.toml) binary target with `test = false` and `bench = false` to resolve host-side workspace test linking errors (`LNK1561`).
- Modified [Database::new](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/database.rs#L48-L56) inside [host-server/src/database.rs](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/database.rs) to run database migrations automatically.
- Removed legacy `create_tables` logic from [host-server/src/database.rs](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/database.rs) and [host-server/src/main.rs](file:///c:/Users/Shaurya/OneDrive/Desktop/projects/qsafe/host-server/src/main.rs).

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
