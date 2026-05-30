# ADR 0007: HSM Serial Interface and Hardware-in-the-Loop Mocking

## Status
Accepted

## Context
The Q-Safe server gateway relies on a dedicated external hardware security module (HSM) implemented on a Raspberry Pi Pico (RP2040) microcontroller for post-quantum key decapsulation and hardware entropy generation.
To develop and run the application across multiple environments (such as developer machines, continuous integration systems, and host environments that do not have physical HSM hardware connected), we must achieve two goals:
1. **Robust Serial Communication**: Establish a packet-framing protocol over USB virtual serial/UART to prevent data alignment corruption or packet truncation.
2. **Local Simulation / Mocking**: Decouple the HSM driver logic from the physical serial port so we can simulate the HSM behavior in memory when physical devices are absent.

## Decision
1. **TLV Packet Framing**: Implement a zero-heap-allocation Type-Length-Value (TLV) packetizer in the shared [qsafe-common](../../common/src/lib.rs) crate. The frame consists of:
   `[Type (1B)] [Length (2B, Big Endian)] [Payload (N Bytes)] [CRC-16-CCITT (2B)]`
2. **Standard Checksum**: Validate all serial transmissions using a **CRC-16-CCITT** polynomial (`0x1021` with init `0xFFFF`), computed over Type, Length, and Payload.
3. **Decoupled Driver Trait**: Define a blocking connection trait `HsmConnection` on the host backend.
4. **HIL Mocking**: Develop an in-memory HSM simulator `MockHsmConnection` that mirrors packet behaviors (generating Kyber keypairs, decapsulating ciphertext using simulated key data, and answering TRNG requests).
5. **Configurable Driver Loading**: Configure the server configuration engine to read `HSM_MOCK` and `HSM_PORT`. If mocking is enabled (default), load `MockHsmConnection`. Otherwise, open the serial port path with `PhysicalHsmConnection`.

## Consequences

### Positive
- **Deterministic Workspace Tests**: The entire host server compiles and passes all unit, integration, and formatting tests offline without needing physical hardware connected.
- **Portability**: The shared `qsafe-common` crate is strictly configured as `#![no_std]`, allowing it to compile for both the x86_64 host server and the thumbv6m firmware target.
- **Resilience**: Packets containing corrupted bytes are automatically discarded on checksum mismatch.

### Negative
- Serial port operations are blocking. The host must wrap connection accesses inside a Mutex (`Arc<Mutex<Box<dyn HsmConnection>>>`) to prevent concurrent port corruption.
