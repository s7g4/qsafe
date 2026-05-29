# System Benchmarks & Metrics: Q-Safe

This document tracks system performance benchmarks, target metrics, and verification limits for both host-side service APIs and embedded HSM operations.

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

## 3. Continuous Integration Verification Targets
- **Code Coverage**: >= 80% coverage across all computational and API modules (tracked via `cargo-tarpaulin`).
- **Static Analysis Compliance**: 0 compiler warnings and 0 Clippy lints (`#[deny(warnings)]`).
