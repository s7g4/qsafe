# Technical Writeup: ML-KEM-768 + X25519 Hybrid Key Exchange

This document describes the hybrid post-quantum key exchange design used by
Q-Safe's crypto core (`host-server/src/crypto.rs`, `handshake.rs`), why it's
built this way, and where the implementation currently stands. It's written
for a technical reader evaluating the design decisions, not as an API
reference (see [API_DOCUMENTATION.md](../API_DOCUMENTATION.md) for that).

**Implementation status**: the primitives described here (ML-KEM
encapsulate/decapsulate, X25519 Diffie-Hellman, HKDF derivation, Ed25519
signing) are implemented and unit-tested in `crypto.rs`. The full
`Handshake` protocol in `handshake.rs` (which composes them into an
Alice/Bob init-respond-finalize exchange with decoy-based eavesdropping
detection) is implemented but **not wired to any HTTP or WebSocket
endpoint** - there is currently no network-reachable handshake. See
[HSM_VERIFICATION_STATUS.md](HSM_VERIFICATION_STATUS.md) for the equivalent
status breakdown of the HSM/hardware side.

## Why hybrid, not just ML-KEM

The threat model driving this design is Harvest-Now-Decrypt-Later (HNDL): an
adversary records encrypted traffic today and decrypts it once a
cryptanalytically-relevant quantum computer exists. A pure ML-KEM design
would defend against that, but ML-KEM (standardized as FIPS 203 in 2024) is
young relative to classical ECDH - CNSA 2.0 and BSI guidance both recommend
running a post-quantum KEM in parallel with a classical algorithm during the
transition period, not in place of it. The reasoning is asymmetric: if a
weakness is later found in the lattice parameters, X25519 keeps the channel
secure; if a weakness is (implausibly) found in Curve25519, ML-KEM keeps the
channel secure. Both must fail for the session key to be compromised.

Q-Safe uses **ML-KEM-768** (Kyber-768) specifically as the middle of NIST's
three parameter sets - it targets AES-192-equivalent security, which is a
reasonable balance of security margin against the payload size and CPU cost
of the two smaller-and-larger options, for a system where key sizes cross
constrained links (USB serial to the HSM, not just the network).

## The two independent key agreements

**ML-KEM-768 (`crypto.rs: generate_kyber_keypair`, `kyber_encapsulate`,
`kyber_decapsulate`)**: a lattice-based KEM. The initiator generates an
ephemeral keypair and sends the public key (1184 bytes); the responder
encapsulates against it, producing a ciphertext (1088 bytes) and a 32-byte
shared secret; the initiator decapsulates the ciphertext with its secret key
to recover the same 32-byte secret. This is the leg intended to survive a
future quantum adversary.

**X25519 (`crypto.rs: generate_x25519_keypair`, `x25519_shared_secret`)**:
classical Diffie-Hellman over Curve25519 (RFC 7748). Both sides generate a
32-byte ephemeral keypair and compute `DH(my_secret, their_public)`, arriving
at the same 32-byte shared secret. This is the leg that's been analyzed for
decades and is not expected to weaken based on any known classical attack.

These two exchanges are independent - the Kyber ciphertext and the X25519
public key travel in the same message (`HandshakeInit` / `HandshakeResponse`
in `handshake.rs`), but neither shared secret depends on the other.

## Combining the secrets: HKDF-SHA3-256

`hybrid_key_agreement` concatenates the two 32-byte shared secrets
(`kyber_ss || x25519_ss`, 64 bytes total) and runs them through
`HKDF-SHA3-256` with the fixed info string `b"qsafe-session-key"`, extracting
a single 32-byte session key. Two design choices worth calling out:

- **No salt** is passed to HKDF-Extract (`Hkdf::new(None, &combined)`). This
  is acceptable here because the "salt" role is effectively played by the
  freshness of both ephemeral keypairs per-session - there's no long-term
  secret being derived that would benefit from a random salt, and both
  inputs are already high-entropy (32 bytes each from an ephemeral KEM/DH).
- **SHA3-256, not SHA2-256**, is the HKDF hash function. This is a
  belt-and-suspenders choice consistent with the project's PQC theme: SHA-3
  is a structurally different construction (sponge, not Merkle-Damgard),
  so a future structural break in SHA-2 wouldn't affect this derivation.

The combined-secret buffer and the session key's stack copies are explicitly
zeroized after use (`combined.zeroize()`, and `HybridSharedSecret` /
`KeyPair` both implement `Zeroize` + `Drop`), so a secret doesn't outlive the
scope that needed it.

## Identity binding and key confirmation

A KEM+DH exchange on its own doesn't prove *who* you're talking to - it's
vulnerable to a man-in-the-middle unless the messages are bound to a known
identity. Each `HandshakeInit`/`HandshakeResponse` carries an **Ed25519
signature** (`identity_sig`) over the serialized message contents, generated
from a per-session identity keypair (`crypto.rs: generate_ed25519_keypair`,
`sign_ed25519`). In a production deployment this identity key would be
long-lived and pinned/verified out-of-band (e.g. via the user's registered
`public_key` in Postgres); today `handshake.rs` generates a fresh identity
keypair per handshake call, which proves the signing/verification mechanics
work but does not yet provide real identity assurance - that wiring
(persisting and verifying a stable identity key per user) is a gap, not
a design decision.

After deriving the session key, both sides compute a **key confirmation
MAC** (`key_confirmation_mac`: `BLAKE3-keyed-hash(BLAKE3(session_key),
direction_label)`, with `"alice-confirmation"` / `"bob-confirmation"` as the
direction label) and the peer recomputes and compares it. This proves both
sides actually derived the identical session key before either one trusts
the channel - a KEM/DH mismatch (corrupted ciphertext, wrong public key,
etc.) is caught here rather than silently producing an unusable channel.

## Decoy-based eavesdropping check

`handshake.rs` also runs a QKD-style decoy-state check
(`qkd.rs: bb84_with_decoy_checks`) alongside the classical/PQC exchange,
committing to decoy bit/basis choices with a BLAKE3 hash
(`decoy_commit`) before revealing them, then computing an observed error
rate. If the error rate exceeds `0.11` (the standard BB84-derived QBER
threshold above which an eavesdropper's presence is assumed), the session
key is zeroized and the handshake aborts (`alice_finalize`). This is a
simulated decoy channel today (`qrng.rs` generates the decoy bits locally,
not over an actual quantum channel) - it exercises the same commit/reveal
and error-rate-threshold logic a real QKD integration would use, but does
not (and cannot, without real quantum hardware) detect a real eavesdropper
on the classical/PQC transport it's layered alongside.

## Why this matters for the HSM boundary

The ML-KEM-768 key sizes (1184-byte public key, 1088-byte ciphertext) are
the reason the TLV framing in `common/` uses a 2-byte length field rather
than a fixed-size frame: the HSM protocol needs to carry payloads meaningfully
larger than the 32-byte X25519 keys or symmetric secrets it also transports.
See [HSM_VERIFICATION_STATUS.md](HSM_VERIFICATION_STATUS.md) for how this
framing is used (and tested) between the host and the Mock HSM, and what's
still missing on the physical RP2040 side.
