# HSM Verification Status

This document states exactly what is proven about the Q-Safe HSM path today,
what is architecturally designed but unverified, and what is not implemented
at all. It exists because earlier project documentation described the system
as "production-ready" / "Shipped" without test coverage to back that up -
this is the conservative correction.

## Summary table

| Layer | Status | Evidence |
|---|---|---|
| TLV packet framing + CRC-16-CCITT (`common/`) | Proven | Used by every test below; shared `#![no_std]` crate compiles for both host and `thumbv6m` targets in CI |
| `HsmConnection` trait + `MockHsmConnection` (host-side) | Proven | [`host-server/tests/hsm_mock_flow.rs`](../host-server/tests/hsm_mock_flow.rs) |
| Register endpoint fetching a real Kyber-768 public key through the HSM abstraction | Proven | Same file, `register_endpoint_fetches_a_real_kyber768_public_key_from_the_mock_hsm` |
| `PhysicalHsmConnection` serial driver (host-side, talks to a real port) | Architecturally sound, unverified | Compiles ([`host-server/src/hardware.rs`](../host-server/src/hardware.rs)) but no test opens a real or virtual serial port against it |
| RP2040 firmware: USB-CDC serial handling, TLV parsing, Kyber decapsulation on-device, QRNG | **Not implemented** | [`firmware/src/main.rs`](../firmware/src/main.rs) is a bare `#![no_std] #![no_main]` binary with only a panic handler. `firmware/Cargo.toml` has zero dependencies - no Embassy, no USB stack, no `pqcrypto-kyber` for the embedded target. |

The last row is the important correction: this isn't "hardware exists but we
haven't run the verification steps yet." The firmware binary that would run
on the RP2040 does not contain any HSM logic yet. Everything the README
describes as "a physically-separate RP2040 acting as a dedicated
cryptographic offloading engine" is, right now, a host-side design
(`PhysicalHsmConnection` + the TLV protocol it speaks) with no corresponding
device-side implementation.

## What is proven

- **Mock HSM protocol surface**: `GetPublicKeyReq` returns a real Kyber-768
  public key; a client that encapsulates against that key and sends the
  resulting ciphertext through `KyberDecapsulateReq` gets back the identical
  shared secret; tampering with the ciphertext yields a different (not
  matching) shared secret; malformed `RandomBytesReq` payloads are rejected.
  See `host-server/tests/hsm_mock_flow.rs`.
- **The HTTP registration path really calls the HSM abstraction**, not a
  stub - proven by exercising `/api/auth/register` over real HTTP against a
  real Postgres database in `host-server/tests/auth_flow.rs` and
  `hsm_mock_flow.rs`.
- **Auth flow**: Argon2id password round-trip, dual-JWT issuance/expiry/
  signature validation, refresh-token rotation, and query-based WebSocket
  token authorization are covered by
  [`host-server/tests/auth_flow.rs`](../host-server/tests/auth_flow.rs) and
  unit tests in [`host-server/src/auth.rs`](../host-server/src/auth.rs).

## What is architecturally sound but unverified

- `PhysicalHsmConnection` (`host-server/src/hardware.rs`): opens a real
  serial port, frames requests with the shared TLV protocol, and parses
  responses with CRC validation. The encode/decode logic it depends on
  (`qsafe-common`) is tested, but the driver itself has never been run
  against a real or simulated serial peer, because there is no peer to talk
  to (see below).

## What is not implemented

- **RP2040 firmware**: no USB-CDC serial stack, no TLV packet handling, no
  on-device Kyber-768 keypair generation/decapsulation, no QRNG sourcing.
  `firmware/src/main.rs` is a 7-line panic-handler stub. `cargo check -p
  qsafe-firmware --target thumbv6m-none-eabi` passes in CI because there is
  nothing to fail - it does not exercise any HSM behavior.

## Exact steps to close the gap (owner-gated - requires physical hardware)

These cannot be completed in this environment; they need a physical RP2040
and a development machine with a USB connection to it.

1. **Write the firmware.** Implement (at minimum) in `firmware/`:
   - An Embassy (or equivalent async embedded runtime) USB-CDC serial
     device that speaks the `qsafe-common` TLV framing.
   - Handlers for `GetPublicKeyReq` (generate/persist a Kyber-768 keypair
     on-device), `KyberDecapsulateReq` (decapsulate in SRAM, zeroize
     immediately after), and `RandomBytesReq` (real hardware entropy, not a
     PRNG).
2. **Flash it** to the RP2040 (e.g. via `probe-rs` or the UF2 bootloader).
3. **Wire the host up to the real device**: set `HSM_MOCK=false` and
   `HSM_PORT=/dev/ttyACM0` (or the platform-appropriate path) in `.env`.
4. **Run the existing integration tests against the physical HSM** instead
   of the mock: the same assertions in `hsm_mock_flow.rs` (public key shape,
   encapsulate/decapsulate round-trip, tamper detection) should be re-run
   with `PhysicalHsmConnection` swapped in, ideally as a `#[ignore]`d test
   gated behind an env var (e.g. `QSAFE_PHYSICAL_HSM_TESTS=1`) so CI doesn't
   require hardware.
5. **Measure and document real latency** for a decapsulation round-trip over
   USB serial (the README currently makes no hardware latency claims -
   any added should cite this measurement).
6. Update the table at the top of this document once each step lands.

Until step 1 exists, "hardware-integrated" in any project description should
be understood as "designed for hardware integration, with a host-side driver
and protocol ready to talk to it" - not "hardware exists and runs this
code."
