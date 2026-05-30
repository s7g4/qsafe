# ADR 0008: Hybrid Cryptography & Memory Zeroization

## Status
Accepted

## Context
When running secure messaging systems, cryptographic keys and intermediate secrets reside in the host's physical RAM. If the application crashes, or if an attacker is able to perform a heap/core dump of the server process (e.g. via system exploits, shared tenancy leakage, or debugging dump tools), any plain private keys or session keys left in memory can be extracted.
To mitigate this threat vector, we must ensure that:
1. All private key pairs and session keys are zeroized immediately after they go out of scope.
2. Intermediate stack-allocated buffers and concatenation structures used during key agreements are actively cleared.

## Decision
1. **Memory Zeroization**: Integrate the `zeroize` crate to clear bytes.
2. **Explicit Zeroization Implementations**:
   - Implement `zeroize::Zeroize` and `Drop` on the `KeyPair` structure in [host-server/src/crypto.rs](host-server/src/crypto.rs), zeroizing the private key vector.
   - Implement `zeroize::Zeroize` and `Drop` on the `HybridSharedSecret` structure in [host-server/src/crypto.rs](host-server/src/crypto.rs), clearing the Kyber shared secret, X25519 shared secret, and the derived session key.
3. **Buffer Hygiene**: Refactor `CryptoEngine` methods to manually zeroize all temporary stack copies or intermediate arrays containing key material (e.g., intermediate stack arrays in `x25519_shared_secret`, `hybrid_key_agreement`, and `sign_ed25519`).

## Consequences

### Positive
- **Active Memory Hygiene**: Key secrets are cleared from heap and stack memory immediately after use, reducing the threat window for memory extraction exploits.
- **Portability**: Leverages the standard Rust compiler drop hierarchy.

### Negative
- Debugging key exchanges using memory dumps becomes harder since secrets are aggressively wiped.
- Slight CPU overhead during zeroization loops (negligible in standard CPU architectures).
