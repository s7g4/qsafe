# Changelog: Q-Safe Gateway

All notable changes to the Q-Safe secure messaging gateway will be documented in this file. This project follows [Semantic Versioning](https://semver.org/).

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
- Restructured the repository into a multi-crate Cargo Workspace containing `host-server/`, `common/`, and `firmware/` crates.
- Sanitized Git history to remove all traces of tracked `.env` credentials.
- Integrated `Config::load()` in `host-server/src/main.rs` to replace raw unvalidated environment fetches.
