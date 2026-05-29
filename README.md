# Q-Safe: Quantum-Safe Messaging Gateway

Q-Safe is a secure messaging gateway built in Rust that integrates post-quantum hybrid cryptography with physical Hardware Security Modules (HSMs). 

Originally built as a student hobby project exploring basic Rust networking and simulated cryptography, this project is being systematically refactored into an industry-grade systems portfolio project, integrating bare-metal Embedded Rust with post-quantum cryptography.

## Project Architecture & Roadmap
The project is organized into a Cargo Workspace:
- `host-server/`: The messaging backend built on Axum, managing WebSocket routing and SQLx storage.
- `firmware/`: Bare-metal Embedded Rust firmware (targeting the RP2040 microcontroller) executing key decapsulations.
- `common/`: Shared Type-Length-Value (TLV) packet definitions compiled for both host and device targets.

### Development Roadmap
- [x] **Project Audit & Security Baseline**: Audit legacy database initialization, connection handling, and cryptographic stubs.
- [x] **Product Vision & Threat Model**: Formulate the threat model (HNDL attacks, host memory vulnerability) and hardware token design.
- [x] **Research Foundation**: Technical study of Module-Lattice KEM (FIPS 203), Embassy async runtime, and serial transmission error correction.
- [x] **Architecture Design**: Decoupling workspace crates and mapping interface sequences.
- [ ] **Database & Config Migration**: Transitioning to SQLx migrations and validated config loaders.
- [ ] **Secure Authentication & Session Lifecycle**: Implementing Argon2id password hashing and access/refresh token dual-flows.
- [ ] **Standardized Hybrid Crypto**: Upgrading Custom Key Agreement to HPKE standards and implementing memory zeroization.
- [ ] **WebSocket Registry**: Rebuilding the async actor registry to route client messages securely.
- [ ] **Telemetry & Observability**: Adding structured JSON logging and Prometheus metrics.

## Documentation Index
- **[PROJECT_AUDIT.md](PROJECT_AUDIT.md)**: Catalog of legacy tech debt, broken stubs, and vulnerabilities.
- **[VISION.md](VISION.md)**: Product positioning, threat modeling, and HSM hardware specifications.
- **[RESEARCH.md](RESEARCH.md)**: Cryptographic algorithms, TLV serial framing, and Embassy runtime specs.
- **[ARCHITECTURE.md](ARCHITECTURE.md)**: High-level architectural specifications, data flows, and hardware integration boundaries.
- **[DEVLOG.md](DEVLOG.md)**: Active engineering journal tracking bugs, root cause analyses, and updates.
