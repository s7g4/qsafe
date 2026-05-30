# Q-Safe: Quantum-Safe Messaging Gateway

Q-Safe is an industry-grade, secure messaging gateway built in Rust that integrates post-quantum hybrid cryptography with physical Hardware Security Modules (HSMs).

Built to defend against Harvest-Now-Decrypt-Later (HNDL) attacks, Q-Safe leverages a zero-trust architecture, strict memory sanitization, and bare-metal RP2040 microcontrollers acting as dedicated cryptographic offloading engines.

## Architecture

The project is organized into a Cargo Workspace spanning host and embedded targets:
- `host-server/`: The messaging backend built on Axum, managing WebSocket routing, PostgreSQL (SQLx) storage, authentication, and HTTP endpoints.
- `firmware/`: Bare-metal Embedded Rust firmware (targeting the RP2040 microcontroller) executing secure key decapsulations and QRNG generation.
- `common/`: Shared Type-Length-Value (TLV) packet definitions compiled for both host and device targets, enabling zero-copy `#[no_std]` serial communication.

## Production-Ready Features

- **Post-Quantum Cryptography**: Module-Lattice KEM (ML-KEM / FIPS 203) integrated with X25519 for hybrid key exchange.
- **Hardware Security Module**: Offloads quantum-safe decapsulation to a physical RP2040 microcontroller via highly-reliable, CRC-checked TLV framing. Includes a software Mock HSM for local development.
- **Strict Authentication**: Argon2id password hashing, Dual-JWT architecture (short-lived access + secure HttpOnly refresh), and query-based WebSocket token authorization.
- **Hardened Security**: Protected against credential stuffing (Rate Limiting via `tower_governor`), memory exhaustion (Bounded Channels), leaky errors (Error Sanitization), and insecure origins (Configurable CORS).
- **Graceful Shutdown**: Zero-drop active request handling during SIGTERM/CTRL+C restarts.
- **Observability**: Exhaustive Prometheus metrics (`/metrics`) tracking latencies, connections, and hardware throughput, paired with `tracing` spans and `x-request-id` headers.
- **Deployment Strategy**: Multi-stage Dockerfile and `docker-compose.yml` for isolated deployment alongside PostgreSQL 16.

## Getting Started

### Local Development (Mock HSM)

1. **Start the database**:
   ```bash
   docker-compose up -d postgres
   ```
2. **Setup Environment**:
   Copy `.env.example` to `.env` and fill in the secrets. Ensure `HSM_MOCK=true`.
3. **Run the API**:
   ```bash
   cargo run -p qsafe-backend
   ```

### Production Deployment

1. Configure `.env` with strong secrets and set `HSM_MOCK=false` with the correct `HSM_PORT` (e.g. `/dev/ttyACM0`).
2. Build and run using Docker Compose:
   ```bash
   docker-compose up -d --build
   ```

## Documentation

- **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)**: Complete REST and WebSocket endpoint specifications for integrating with the gateway.
- **[ARCHITECTURE.md](ARCHITECTURE.md)**: High-level architectural specifications, data flows, hardware integration boundaries, and observability metrics.
- **[CHANGELOG.md](CHANGELOG.md)**: Semantic version tracking and updates record.
- **[METRICS.md](METRICS.md)**: Latency limits, memory zeroization parameters, binary overhead budgets, testing strategy, and CI/CD pipelines.
- **[docs/adr/](docs/adr/)**: Architectural Decision Records (ADRs) tracking design updates and transitions.
- **[DEVLOG.md](DEVLOG.md)**: Active engineering journal tracking bugs, root cause analyses, and updates.
