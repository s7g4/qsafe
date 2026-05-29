# Research Foundation: Q-Safe Gateway

As I prepare to transition Q-Safe from software simulation to a hardware-integrated system, I need to establish the core protocols, algorithms, and libraries that will bridge the host server and the embedded HSM. This document catalogs my research in cryptography, serial communications, and bare-metal firmware design.

## 1. Hybrid Post-Quantum Cryptography (PQC)

### Core Algorithms
- **ML-KEM (FIPS 203)**: Formerly known as CRYSTALS-Kyber, this is a module lattice-based key encapsulation mechanism selected by NIST. I am choosing Kyber-768/ML-KEM-768 to balance execution speed, ciphertext payload size (1088 bytes), and security margin.
- **X25519 (RFC 7748)**: A classical Elliptic Curve Diffie-Hellman (ECDH) key exchange over Curve25519.
- **Hybrid Construction**: Because ML-KEM is a relatively new standard, standard security recommendations (like those from CNSA and BSI) require running it in parallel with classical Curve25519. If a cryptographic weakness is discovered in the lattice parameters, the channel remains secured by Curve25519.

### Key Questions
- How do we derive a unified session key from both shared secrets? I will use HKDF-SHA3-256 to combine the 32-byte X25519 output and the 32-byte ML-KEM output.
- What is the size impact on the network handshake? The public key sizes increase from 32 bytes (X25519) to over 1100 bytes (Kyber-768 public key + X25519 public key). The API payloads must accommodate this size jump.


## 2. Serial Protocol & TLV Framing

Because the host server and the micro-controller communicate over a raw byte stream (USB virtual serial/UART), we need a packet framing format to prevent data truncation or alignment corruption.

### Frame Layout (Type-Length-Value)
Every USB transmission will use a strict packet structure:
```
+------------+------------+-----------------+-----------------------+
|  Type (1B) | Length(2B) |   Payload (NB)  |    CRC-16 Check(2B)   |
+------------+------------+-----------------+-----------------------+
```

### Packet Operations
- **`0x01` / `0x02` (Random Bytes Request/Response)**: Host requests true hardware entropy from the microcontroller for cryptographic seeds.
- **`0x03` / `0x04` (Kyber Decapsulate Request/Response)**: Host sends the Kyber Ciphertext (CT). Device decapsulates it in SRAM and returns the derived shared secret.

### Error Detection
Raw UART/USB serial connections are prone to transmission errors. I will use a **CRC-16-CCITT** checksum appended to the end of every packet. On checksum failure, the receiver discards the corrupt frame and requests a retransmission.

---

## 3. Embedded Firmware Architecture (Embassy)

### The Runtime
For the microcontroller firmware, I am using **Embassy**, an asynchronous, bare-metal Rust framework for embedded systems.
- **Why Embassy?**: Embassy allows writing low-level code using Rust's `async/await` syntax. Instead of spinning CPU cycles in a polling loop waiting for USB transactions, the microcontroller suspends and handles events via hardware interrupts, lowering power usage and simplifying concurrency.
- **Target Microcontroller**: Raspberry Pi Pico (RP2040). It features dual ARM Cortex-M0+ cores, 264KB SRAM, and native USB controllers.

---

## 4. Hardware-in-the-Loop (HIL) Software Simulation

To test the system without requiring physical hardware connected to the host at compile time, I will design a **mock serial device driver**:
- During testing, the host-side serial module will bypass the physical port and connect to a local software loopback.
- This loopback will mock the microcontroller's responses, calculating simulated Kyber operations in memory and validating packet framing, error recovery, and CRC validations.
