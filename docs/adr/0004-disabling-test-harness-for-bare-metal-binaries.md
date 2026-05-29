# ADR 0004: Disabling Host-Side Test Harness for Bare-Metal Binaries

## Status
Accepted

## Context
When running cargo commands (such as `cargo test` or `cargo check --all-targets`) at the workspace level, Cargo attempts to build unit-test binaries for all crates and all targets. 
However, the `qsafe-firmware` crate is configured as a `#![no_std]` and `#![no_main]` binary. 
Because it lacks a standard runtime environment and does not define a standard host-compatible entry point, the host linker (`link.exe` on Windows) fails during the test build stage with:
`fatal error LNK1561: entry point must be defined`

## Decision
Configure `firmware/Cargo.toml` to disable the standard test and benchmark harness compilation for the main firmware binary:
```toml
[[bin]]
name = "qsafe-firmware"
path = "src/main.rs"
test = false
bench = false
```

Any firmware unit/integration tests must either be written inside a separate library/host-compatible mock library or validated using custom target-specific hardware-in-the-loop (HIL) test suites.

## Consequences

### Positive
- **Clean Workspace Verification**: Commands like `cargo test` and `cargo clippy --workspace --all-targets` now execute and pass successfully at the workspace root without target linker failures.
- **Clear Decoupling**: Separation of host-side tests and bare-metal embedded binaries is preserved.

### Negative
- None. Host-side simulation testing can still be performed on target-agnostic components (e.g., using `qsafe-common` or mock libraries).
