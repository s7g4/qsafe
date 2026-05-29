# ADR 0003: Static Secret & Signature Type Unification

## Status
Accepted

## Context
During the workspace compilation check, we identified three critical type safety and compilation errors in the legacy cryptographic and communications codebases:
1. **Unserializable Ephemeral Keys**: The legacy handshake protocol required serializing Bob's X25519 private key to disk to complete the session exchange. However, the code used `EphemeralSecret` which purposely disables serialization to prevent key reuse.
2. **Typing Misalignments**: The legacy messaging and handshake modules attempted to import `ed25519_dalek::Signature` but instantiated it using fields defined inside our custom `QSafeSignature` wrapper, causing compilation errors.
3. **Rust Compiler Deprecations**: The bare-metal firmware target binary failed to compile because Cargo defaults to unwinding panics, which is unsupported on non-std embedded targets.

## Decision
1. **Enable Reusable Secrets**: We will configure `x25519-dalek` with the `static_secrets` feature enabled in `host-server/Cargo.toml`. This allows using `StaticSecret`, which supports serialization (`to_bytes` / `from`) for handshake persistence.
2. **Unify Signature Wrappers**: We will replace all misaligned instances of `Signature` inside `handshake.rs` and `messaging.rs` with `QSafeSignature`.
3. **Bare-Metal Panic Strategy**: We will configure the entire Cargo workspace to use `panic = "abort"` profiles inside `Cargo.toml`.

## Consequences

### Positive
- **Successful Bare-Metal Compilation**: The firmware compiles cleanly for the RP2040 target core.
- **Type Safety**: Unifying our custom `QSafeSignature` eliminates compiler typing errors and visibility issues across the service boundaries.

### Negative
- **Key Reuse Risks**: Using `StaticSecret` enables key serialization but requires us to ensure that session keys are zeroized from memory immediately after consumption to prevent leakage.
