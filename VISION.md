# Project Vision: Q-Safe Gateway

I originally built Q-Safe as a student project to explore Rust web APIs and basic cryptographic key exchange simulations. While the initial prototype worked at a basic level, it had massive architectural limitations and relied entirely on simulated security. 

Now, as I prepare for professional roles in systems software and embedded engineering, I am using this codebase as a playground to demonstrate how to transition a hobby codebase into a production-ready, hardware-secured gateway.

## What Problem Does This Solve?
Most secure messaging backends store keys and decrypt payloads directly in the host server's memory. If a server is compromised at the OS level (via kernel exploits or memory scraping), all active session keys and identity secrets are leaked.

Furthermore, traditional key exchange protocols (like ECDH or RSA) are vulnerable to future quantum computer attacks. Adversaries are actively capturing encrypted traffic today (Harvest Now, Decrypt Later) to decrypt it once quantum scaling is achieved.

Q-Safe addresses both issues:
1. **Hybrid Cryptography**: It runs Curve25519 (classical) and ML-KEM-768 (post-quantum) in parallel to protect data against retro-active quantum decryption.
2. **Hardware Security Module (HSM) Isolation**: It moves private key storage and Kyber decapsulation out of the host's memory onto an external micro-controller (like an RP2040) running bare-metal Rust. The host never sees the private key; it only sends ciphertexts over USB and receives derived shared secrets.

## Target Audience & Portfolio Goals
This project is designed to show systems architects that I can build software across the entire systems stack:
- **Low-Level Firmware**: Designing bare-metal Rust drivers and parsing USB packet streams on a microcontroller.
- **Systems Engineering**: Implementing custom protocol framing (TLV), memory safety (zeroization), and concurrency management in Rust.
- **Backend Architecture**: Managing state routing, database migrations, and real-time WebSocket communication in a multi-threaded server.

## What Makes it Unique?
Instead of just building another standard web backend or a disconnected firmware stub, this project integrates the two. It connects a real-time HTTP/WebSocket server to a hardware security token (simulated locally for automated testing), showing a clean, end-to-end security architecture.
