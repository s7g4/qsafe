# Developer Log: Q-Safe

## 2026-05-29: Initial Repository Audit & Git Sanitization

### Goal
Audit the legacy Q-Safe codebase, identify security and architectural debt, and sanitize the repository's Git history to ensure zero leaks of environment credentials.

### Work Completed
- Sanitized remote history: Force-pushed the local sanitized master commit to GitHub, purging the `.env` commit history globally.
- Expired local reflogs and triggered aggressive Git garbage collection (`git gc`) to delete reference dangling objects from the local workspace.
- Created `PROJECT_AUDIT.md` evaluating the state of the backend APIs, the broken WebSocket mapper, database table initialization, security debt, and technical risks.
- Formulated the student refactoring narrative: framing the codebase evolution from a student learning prototype to a professional systems/embedded showcase project.

### Problems Encountered
- Local branch diverged from origin/master due to local commits being amended.
- Resolved by performing a force-push (`git push -f origin master`) to rewrite the GitHub remote history.

### Lessons Learned
- Sanitizing credentials early is critical to maintaining developer credibility and project security.
- Restructuring a project incrementally in Git mirrors professional, research-driven engineering methodologies.

### Metrics
- **Files Created**: 2 (`PROJECT_AUDIT.md`, `DEVLOG.md`).
- **Files Updated**: 1 (`README.md`).
- **Code Changes**: 0 lines modified.

## 2026-05-29: Product Vision & Threat Model Definition

### Goal
Define the strategic direction of the project, framing it as a refactoring case study that bridges web backends and embedded hardware modules.

### Work Completed
- Authored `VISION.md` detailing the transition from a student simulation to a hardware-assisted messaging gateway.
- Clarified the target problem (Harvest Now, Decrypt Later and host memory vulnerability) and the integration of the USB HSM token.

### Metrics
- **Files Created**: 1 (`VISION.md`).
- **Code Changes**: 0 lines.

## 2026-05-29: Cryptographic & Embedded Protocol Research

### Goal
Document the technical specifications for the hybrid key exchange, serial packet framing layout, and the embedded asynchronous runtime environment.

### Work Completed
- Researched ML-KEM-768 payload sizes and defined the HKDF-SHA3-256 derivation scheme for the hybrid session key.
- Designed the Type-Length-Value (TLV) serial packet structure with CRC-16-CCITT validation checks.
- Formulated the Embassy framework target setup for the Raspberry Pi Pico (RP2040) firmware architecture.

### Metrics
- **Files Created**: 1 (`RESEARCH.md`).
- **Code Changes**: 0 lines.

## 2026-05-29: System Architecture & Cargo Workspace Redesign

### Goal
Define the decoupled system architecture boundaries, communication sequence protocols, and structural layouts for the multi-crate Cargo Workspace.

### Work Completed
- Authored `ARCHITECTURE.md` detailing the workspace directory boundaries (`host-server`, `firmware`, `common`).
- Modeled the system topology showing API-to-WebSocket transitions, hybrid cryptographic engines, and the USB serial connection to the RP2040 microcontroller.
- Designed the handshake communication sequence over USB-CDC using a sequence flow diagram.
- Cataloged hardware transmission failure mitigation metrics (CRC-16 validation and software fallbacks).

### Metrics
- **Files Created**: 1 (`ARCHITECTURE.md`).
- **Code Changes**: 0 lines.

## 2026-05-29: Development Roadmap Definition

### Goal
Define the sequential engineering milestones, success criteria, and deliverables for the refactoring process.

### Work Completed
- Authored `ROADMAP.md` mapping out 6 key milestones (from config/migrations refactoring to telemetry instrumentation).
- Established baseline expected commits, risks, and success metrics for each milestone.

### Metrics
- **Files Created**: 1 (`ROADMAP.md`).
- **Code Changes**: 0 lines.

## 2026-05-29: Setup of the Project Documentation & Standard Systems

### Goal
Establish the unified documentation layout, metrics logs, change tracking records, and the Architectural Decision Record (ADR) history register.

### Work Completed
- Created `CHANGELOG.md` initializing the SemVer tracking record.
- Created `METRICS.md` defining latency thresholds, memory hygiene benchmarks, and target test coverages.
- Created the first Architectural Decision Record (`docs/adr/0001-cargo-workspace-and-hsm-architecture.md`) documenting the rationale behind the multi-crate Cargo Workspace and embedded integration.

### Metrics
- **Files Created**: 3 (`CHANGELOG.md`, `METRICS.md`, `docs/adr/0001-cargo-workspace-and-hsm-architecture.md`).
- **Code Changes**: 0 lines.

## 2026-05-29: Observability Specifications & Testing Strategy Formulation

### Goal
Define the monitoring framework and testing strategy blueprints to measure system stability and verify cryptographic correctness.

### Work Completed
- Integrated **Observability Specs** directly into `ARCHITECTURE.md`, outlining the `tracing` JSON logging framework, Prometheus metrics, and downstream status indicators in `/api/health`.
- Integrated the **Testing Strategy** directly into `METRICS.md`, detailing unit testing bounds, database rollbacks, integration routes, and HIL simulation mock checks.

### Metrics
- **Files Updated**: 2 (`ARCHITECTURE.md`, `METRICS.md`).
- **Code Changes**: 0 lines.

## 2026-05-29: Continuous Integration & Automated Verification Setup

### Goal
Define the automated CI/CD stages, security scanning boundaries, and cross-compilation target compilation checks.

### Work Completed
- Integrated **CI/CD Specifications** directly into `METRICS.md` under Section 3, outlining checks for formatting (`rustfmt`), lints (`clippy`), security audits (`cargo audit`), duplicate/license controls (`cargo deny`), and target compilation bounds (host x86_64 vs. embedded thumbv6m targets).

### Metrics
- **Files Updated**: 1 (`METRICS.md`).
- **Code Changes**: 0 lines.

## 2026-05-29: Implementing Validated Config Loader

### Goal
Implement a centralized and validated configuration engine to replace raw environment variable lookups.

### Work Completed
- Added `pub mod config;` to `host-server/src/lib.rs`.
- Created `host-server/src/config.rs` implementing a type-safe `Config` struct.
- Modified `host-server/src/main.rs` to read parameters from `Config::load()` and handle boot errors gracefully.

### Metrics
- **Files Modified**: 2 (`host-server/src/lib.rs`, `host-server/src/main.rs`).
- **Files Created**: 1 (`host-server/src/config.rs`).
- **Panics Avoided**: Removed 3 unhandled `unwrap()` / `expect()` calls on environment variable lookups.

## 2026-05-29: Resolving Workspace Compilation Errors & Warnings

### Goal
Resolve all compile errors and compiler warnings in the workspace to restore a clean build baseline.

### Work Completed
- Enabled the `static_secrets` feature in `host-server/Cargo.toml` for `x25519-dalek` and refactored `crypto.rs` to use `StaticSecret` instead of `EphemeralSecret` to support serializing keys during handshakes.
- Unified signature structs across the workspace, changing misaligned `Signature` typings in `handshake.rs` and `messaging.rs` to `QSafeSignature`.
- Replaced `SigningKey::generate` with a custom, feature-independent key loading scheme in `crypto.rs`.
- Fixed double-mutable borrows of the QRNG simulator in `qkd.rs` and use-after-move vectors in `qrng.rs`.
- Resolved all unused imports, variables, and dead code warnings inside `main.rs`, `auth.rs`, `handshake.rs`, and `messaging.rs`.
- Created `docs/adr/0003-static-secret-and-signature-type-unification.md` documenting these changes.

### Metrics
- **Files Modified**: 8 (`host-server/Cargo.toml`, `host-server/src/main.rs`, `host-server/src/auth.rs`, `host-server/src/crypto.rs`, `host-server/src/handshake.rs`, `host-server/src/messaging.rs`, `host-server/src/qkd.rs`, `host-server/src/qrng.rs`).
- **Files Created**: 1 (`docs/adr/0003-static-secret-and-signature-type-unification.md`).
- **Compiler Warnings Resolved**: 13 warnings eliminated.
- **Build Status**: 100% clean compilation.

## 2026-05-29: Resolving Workspace Binary Test Linking Errors

### Goal
Resolve workspace-level host-side linking failures when compiling tests for bare-metal firmware binaries.

### Work Completed
- Added custom `[[bin]]` configuration to `firmware/Cargo.toml` with `test = false` and `bench = false` targets to prevent host-side test harness generation for the `#![no_main]` embedded binary.
- Created `docs/adr/0004-disabling-test-harness-for-bare-metal-binaries.md` documenting this solution.
- Verified compilation and testing flow across all workspaces by running clean checks and test suites.

### Metrics
- **Files Modified**: 1 (`firmware/Cargo.toml`).
- **Files Created**: 1 (`docs/adr/0004-disabling-test-harness-for-bare-metal-binaries.md`).
- **Build Status**: 100% clean verification pass for workspace-level tests.

## 2026-05-29: Implementing SQLx Database Migrations

### Goal
Transition database schema definition and initialization to compilation-guaranteed SQLx migrations.

### Work Completed
- Extracted inline SQL schema creation queries into a versioned migration: [0001_init.sql](host-server/migrations/0001_init.sql).
- Modified [Database::new](host-server/src/database.rs#L48-L56) to execute `sqlx::migrate!().run(&pool).await?` on startup.
- Removed legacy `create_tables` logic from [database.rs](host-server/src/database.rs) and [main.rs](host-server/src/main.rs).
- Documented design decisions in [docs/adr/0005-database-migrations.md](docs/adr/0005-database-migrations.md).
- Verified successful workspace compilation and test suite run.

### Metrics
- **Files Modified**: 2 (`host-server/src/database.rs`, `host-server/src/main.rs`).
- **Files Created**: 2 (`host-server/migrations/0001_init.sql`, `docs/adr/0005-database-migrations.md`).
- **Build Status**: 100% clean compilation.

## 2026-05-30: Secure Authentication & Error Handling Framework

### Goal
Upgrade password hashing to Argon2id, introduce a dual-token (Access Token + HttpOnly Cookie Refresh Token) session rotation system, and implement panic-free structured JSON error propagation.

### Work Completed
- Added `argon2` and `thiserror` dependencies and removed `bcrypt` in [host-server/Cargo.toml](host-server/Cargo.toml).
- Created [host-server/src/error.rs](host-server/src/error.rs) defining the custom `QSafeError` enum mapping errors to structured JSON response payloads.
- Registered the new error module inside [host-server/src/lib.rs](host-server/src/lib.rs).
- Refactored [host-server/src/auth.rs](host-server/src/auth.rs) to use Argon2id for hashing/verification and generate separate Access (15m) and Refresh (7d) JWT tokens.
- Refactored handlers in [host-server/src/main.rs](host-server/src/main.rs) to eliminate all unhandled `unwrap()` calls, return rotated refresh token cookies (`Set-Cookie`) on register/login/refresh, and clear cookies on logout.
- Created [docs/adr/0006-secure-authentication-and-error-handling.md](docs/adr/0006-secure-authentication-and-error-handling.md) documenting design choices.
- Verified workspace builds, formatting, clippy static analysis, and test suites.

### Metrics
- **Files Modified**: 4 (`host-server/Cargo.toml`, `host-server/src/auth.rs`, `host-server/src/lib.rs`, `host-server/src/main.rs`).
- **Files Created**: 2 (`host-server/src/error.rs`, `docs/adr/0006-secure-authentication-and-error-handling.md`).
- **Panics Avoided**: 0 unhandled `unwrap()` and `expect()` remaining in handler endpoints.
- **Build Status**: 100% clean check, format, clippy, and test pass.

## 2026-05-30: HSM Serial Connection & HIL Simulator

### Goal
Implement host-side serial communication driver and a local mock responder to enable offline testing without physical microcontrollers.

### Work Completed
- Turn the shared [common/src/lib.rs](common/src/lib.rs) workspace member into a `#![no_std]` crate. Added packet type representations (`0x01`–`0x06`), CRC-16-CCITT validation checks, and encoding/decoding loops.
- Added `serialport` and `qsafe-common` dependency declarations inside [host-server/Cargo.toml](host-server/Cargo.toml).
- Created [host-server/src/hardware.rs](host-server/src/hardware.rs) implementing the `HsmConnection` trait, physical serial driver, and in-memory mock simulator. Registered the module in [host-server/src/lib.rs](host-server/src/lib.rs).
- Modified [host-server/src/config.rs](host-server/src/config.rs) and [host-server/src/main.rs](host-server/src/main.rs) to wire the `AppState` with mock vs physical connection initializers.
- Refactored registration endpoint in `main.rs` to fetch Kyber public keys directly from the HSM simulator/connection.
- Documented decisions in [docs/adr/0007-hardware-interface-driver-and-simulation.md](docs/adr/0007-hardware-interface-driver-and-simulation.md).
- Verified workspace builds, formatting, clippy static analysis, and test suites.

### Metrics
- **Files Modified**: 5 (`common/src/lib.rs`, `host-server/Cargo.toml`, `host-server/src/config.rs`, `host-server/src/lib.rs`, `host-server/src/main.rs`).
- **Files Created**: 2 (`host-server/src/hardware.rs`, `docs/adr/0007-hardware-interface-driver-and-simulation.md`).
- **Build Status**: 100% clean check, format, clippy, and test pass.

## 2026-05-30: Hybrid Cryptography & Memory Zeroization

### Goal
Implement standardized memory zeroization routines for sensitive key pairs, shared secrets, and intermediate stack buffers inside `CryptoEngine`.

### Work Completed
- Added `zeroize` trait imports and implemented `Zeroize` and `Drop` on `KeyPair` and `HybridSharedSecret` inside [host-server/src/crypto.rs](host-server/src/crypto.rs) to clear secret fields on scope exit.
- Refactored `x25519_shared_secret`, `hybrid_key_agreement`, `generate_x25519_keypair`, `generate_ed25519_keypair`, and `sign_ed25519` to actively clear stack keys, temporary buffers, and concatenated array allocations.
- Refactored [host-server/src/handshake.rs](host-server/src/handshake.rs) to clone key parameters before structuring payloads, avoiding moves out of types that implement `Drop`.
- Created [docs/adr/0008-hybrid-crypto-standards-and-memory-zeroization.md](docs/adr/0008-hybrid-crypto-standards-and-memory-zeroization.md) documenting decisions.
- Added `libudev-dev` installation to [.github/workflows/ci.yml](.github/workflows/ci.yml) to fix CI/CD runner compilation errors for the `serialport` crate.
- Verified workspace builds, formatting, clippy static analysis, and test suites.

### Metrics
- **Files Modified**: 3 (`host-server/src/crypto.rs`, `host-server/src/handshake.rs`, `.github/workflows/ci.yml`).
- **Files Created**: 1 (`docs/adr/0008-hybrid-crypto-standards-and-memory-zeroization.md`).
- **Build Status**: 100% clean check, format, clippy, and test pass.






