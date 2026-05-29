# ADR 0001: Cargo Workspace & HSM Architecture Integration

## Status
Proposed

## Context
The legacy application was built as a single monolithic Rust crate containing the web server logic, database client code, and simulated cryptography routines in a single target binary. 

To support the strategic goal of developing Q-Safe into an industry-grade portfolio piece demonstrating embedded systems engineering, we must integrate a bare-metal microcontroller firmware module. This module will run isolated cryptographic routines. We need to:
1. Compile firmware targeting ARM Cortex-M0+ (RP2040) microcontrollers.
2. Compile host backend services targeting standard x86_64 server architectures.
3. Share command structure definitions and packet framing layouts directly between the two architectures to avoid protocol serialization desynchronization.

## Decision
We will transition the repository architecture to a **Cargo Workspace**. 

The codebase will be split into three decoupled crates:
1. **`host-server` (Binary Crate)**: Axum-based web gateway and messaging core orchestrating database persistence and client connections (refactored from the legacy codebase).
2. **`firmware` (Binary Crate)**: Bare-metal embedded Rust application running the async `embassy` runtime for the target ARM microcontroller.
3. **`common` (Library Crate)**: Shared packet payloads, enum representations of message types, and serialization functions compiled for both target architectures.

Additionally, to allow local developer testing without requiring physical development boards, the `host-server`'s serial interface driver will implement a **Hardware-in-the-Loop (HIL) software simulator**. This simulator mocks serial transmissions on the host and feeds them back into local cryptographic loops during integration test runs.

## Consequences

### Positive
- **Arch-Level Isolation**: Private keys and decapsulation steps are restricted to the physical or simulated SRAM of the microcontroller, representing true HSM engineering.
- **Compile-Time Protocol Safety**: Direct reuse of types inside the `common` crate ensures that modifications to payload definitions automatically flag compile-time errors in both the host code and firmware code if mismatched.
- **Robust Integration Testing**: The software hardware-mock driver enables automated testing of serial protocol framing and edge-case handling in standard CI/CD pipelines.

### Negative / Overhead
- **Toolchain Requirements**: Developers must install the target compilation tools (e.g., `rustup target add thumbv6m-none-eabi`) to build the entire workspace.
- **Workspace Settings**: Increase in compilation configurations and shared settings under the root workspace `Cargo.toml`.
