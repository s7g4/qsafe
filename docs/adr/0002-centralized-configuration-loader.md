# ADR 0002: Centralized Configuration Loader

## Status
Accepted

## Context
The messaging gateway requires several runtime parameters (PostgreSQL database credentials, JWT secret keys, and server TCP ports) to boot. In the legacy codebase, these variables were fetched directly inside the execution threads using `std::env::var().expect()` calls. 

This model introduced two operational issues:
1. **Runtime Panics**: If an environment key was missing, or if the port was formatted as an invalid integer, the thread would panic and crash the service.
2. **Scatter Dependencies**: Multiple files had direct dependencies on the system environment state, complicating testing and validation.

## Decision
We will centralize all configurations into a type-safe `Config` struct defined in `host-server/src/config.rs`.

- The struct will parse environment values during application boot, converting strings to appropriate types (such as `u16` for port bounds).
- It will return structured errors (`Result`) instead of panicking, enabling the entry point `main.rs` to log config failures and exit cleanly.
- To avoid adding dependencies that are not cached in the local environment, the parser will use standard library matches and `dotenvy`.

## Consequences

### Positive
- **Deterministic Boot**: All configuration errors are caught immediately at startup, preventing mid-runtime crashes.
- **Improved Testability**: We can easily mock configurations by initializing the `Config` struct directly in unit and integration tests without injecting environment variables into the host OS.

### Negative
- **Boilerplate**: Adding new variables requires updating the struct fields and manual parsing logic.
