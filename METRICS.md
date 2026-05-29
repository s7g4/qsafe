# System Benchmarks & Metrics: Q-Safe

This document tracks system performance benchmarks, target metrics, verification limits, testing strategies, and CI/CD parameters for both host-side service APIs and embedded HSM operations.

## 1. Latency & Connection Targets

| Component / Operation | Metric Measured | Baseline (Hobby) | Target (Production) | Verification Command |
| :--- | :--- | :--- | :--- | :--- |
| **API Server Startup** | Boot latency | ~1.5s (due to sync DB queries) | **< 100ms** (validated configs, pooled connections) | `cargo run` timestamp logs |
| **HTTP Register/Login** | Endpoint response time | ~500ms (unprofiled bcrypt) | **< 150ms** (Argon2id configured for interactive profiles) | `wrk -t4 -c20 -d10s /api/auth/login` |
| **WS Frame Dispatch** | Client-to-client message routing | N/A (echo stub) | **< 5ms** (async channel-based dispatch) | Integration bench tests |
| **Hybrid Key Decapsulate** | Time to decapsulate Kyber on host | ~2ms (software compute) | **< 3ms** (optimized pqcrypto compilation) | `cargo bench` |
| **Hardware Key Decapsulate** | Time to decapsulate Kyber on device (M0+) | N/A (unimplemented) | **< 200ms** (133MHz cortex-m0+) | Internal hardware firmware timer logs |

## 2. Memory & Payload Metrics

- **Private Key Memory Hygiene**: 100% of generated private key buffers (`KyberSecretKey`, `EphemeralSecret`, `SigningKey`) must be cleared from memory immediately on drop. 
  * *Verification*: Monitored via heap memory sanitizers and memory profiling.
- **Embedded Binary Overhead**: The compiled bare-metal Rust firmware for the RP2040 micro-controller must fit within typical target constraints:
  * **SRAM Consumption**: < 64KB (target: <= 32KB).
  * **Flash Binary Size**: < 256KB.
- **Docker Image Footprint**: 
  * **Production Image Size**: < 50MB (achieved using multi-stage scratch/alpine builds).

## 3. Continuous Integration & Pipeline Automation

To maintain code health and prevent integration regression, the automated CI pipeline (e.g., GitHub Actions) enforces strict checks on every pull request to the `master` branch:

### Stage 1: Style & Code Quality Check
- **Code Formatting**: Runs `cargo fmt --all -- --check` to ensure the codebase strictly complies with standard style rules.
- **Static Analysis (Lints)**: Runs `cargo clippy --workspace --all-targets -- -D warnings` to catch common code smells, suboptimal patterns, or API misuses.

### Stage 2: Security Auditing
- **Dependency Scan**: Runs `cargo audit` to verify that none of the third-party crates defined in `Cargo.lock` contain active vulnerabilities logged in the Rust Advisory Database.
- **License & Dependency Control**: Runs `cargo deny check` to block dependencies using incompatible open-source licenses or adding duplicate crates.

### Stage 3: Compilation & Cross-Build Verification
- **Host Compilation**: Runs `cargo check --workspace --bins --tests --all-targets` to verify successful x86_64 host compilation.
- **Embedded Compilation**: Runs `cargo check -p qsafe-firmware --target thumbv6m-none-eabi` to ensure the microcontroller firmware builds successfully without std library dependencies.

### Stage 4: Automated Testing & Coverage
- **Test Execution**: Runs `cargo test --workspace --all-targets` to execute all unit and integration test blocks.
- **Coverage Guard**: Runs `cargo tarpaulin --workspace --out Xml` to verify that total codebase coverage remains above **80%**.

## 4. Testing & Verification Strategy

We implement a multi-tiered testing strategy to guarantee security, protocol accuracy, and stability across the workspace.

### Unit Testing
- **Cryptographic Engine**: Unit tests inside `host-server/src/crypto.rs` verify key generation, encapsulation, signatures, and decapsulation algorithms.
- **State Parsers & Utilities**: Tests inside the `common` library crate verify TLV serial parsing, binary packet serialization, and CRC-16 computation limits.
- **Mock DB Transactors**: Database unit tests utilize SQLx's test transactor macro (`#[sqlx::test]`), wrapping every test in a transaction that rolls back on completion to keep local databases clean.

### Integration Testing
- **API Flow verification**: Integration test suites inside `host-server/tests/` spin up a test instance of the Axum service using temporary DB configurations to run full registration, login, and token rotation pipelines.
- **WebSocket Route Tests**: Test scripts spawn multiple async tasks running WebSocket client streams, verifying frame delivery guarantees, offline queues, and message dispatching latencies.

### Hardware-in-the-Loop (HIL) Simulation Tests
- **Mock Serial Gateway**: To run integrations without physical micro-controllers, the host-side hardware module integrates a virtual serial mock.
- **Edge-Case Simulation**: The simulator mocks physical network drops, corrupt CRC-16 packets, and timeouts. This verifies that the host serial driver falls back to software engines safely under device failures.
