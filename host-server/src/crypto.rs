//! Cryptographic primitives for Q-Safe

use blake3;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use pqcrypto_kyber::kyber768::{decapsulate, encapsulate, keypair};
use pqcrypto_traits::kem::{
    Ciphertext as KyberCiphertext, PublicKey as KyberPublicKey, SecretKey as KyberSecretKey,
    SharedSecret as KyberSharedSecret,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha3::Sha3_256;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}
impl Zeroize for KeyPair {
    fn zeroize(&mut self) {
        self.secret_key.zeroize();
    }
}
impl Drop for KeyPair {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QSafeSignature {
    pub signature: Vec<u8>,
    pub message: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HybridSharedSecret {
    pub kyber_ss: Vec<u8>,
    pub x25519_ss: Vec<u8>,
    pub session_key: Vec<u8>,
}
impl Zeroize for HybridSharedSecret {
    fn zeroize(&mut self) {
        self.kyber_ss.zeroize();
        self.x25519_ss.zeroize();
        self.session_key.zeroize();
    }
}
impl Drop for HybridSharedSecret {
    fn drop(&mut self) {
        self.zeroize();
    }
}

pub struct CryptoEngine {
    rng: rand::rngs::OsRng,
}

#[allow(clippy::new_without_default)]
impl CryptoEngine {
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::OsRng,
        }
    }

    /// Generate a post-quantum key pair using Kyber
    pub fn generate_kyber_keypair(&mut self) -> Result<KeyPair, Box<dyn std::error::Error>> {
        let (pk, sk) = keypair();
        Ok(KeyPair {
            public_key: pk.as_bytes().to_vec(),
            secret_key: sk.as_bytes().to_vec(),
        })
    }

    /// Perform post-quantum key encapsulation with Kyber
    pub fn kyber_encapsulate(
        &mut self,
        public_key: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        let pk = KyberPublicKey::from_bytes(public_key)?;
        let (ss, ct) = encapsulate(&pk);
        Ok((ss.as_bytes().to_vec(), ct.as_bytes().to_vec()))
    }

    /// Perform post-quantum key decapsulation with Kyber
    pub fn kyber_decapsulate(
        &mut self,
        secret_key: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let sk = KyberSecretKey::from_bytes(secret_key)?;
        let ct = KyberCiphertext::from_bytes(ciphertext)?;
        let ss = decapsulate(&ct, &sk);
        Ok(ss.as_bytes().to_vec())
    }

    /// Generate ephemeral X25519 keypair
    pub fn generate_x25519_keypair(&mut self) -> Result<KeyPair, Box<dyn std::error::Error>> {
        let secret = StaticSecret::random_from_rng(self.rng);
        let public = PublicKey::from(&secret);
        let mut secret_bytes = secret.to_bytes();
        let keypair = KeyPair {
            public_key: public.to_bytes().to_vec(),
            secret_key: secret_bytes.to_vec(),
        };
        secret_bytes.zeroize();
        Ok(keypair)
    }

    /// Compute X25519 shared secret
    pub fn x25519_shared_secret(
        &self,
        secret_key: &[u8],
        public_key: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut secret_bytes = <[u8; 32]>::try_from(secret_key)?;
        let secret = StaticSecret::from(secret_bytes);
        secret_bytes.zeroize(); // Clear the stack copy of private bytes
        let public_bytes = <[u8; 32]>::try_from(public_key)?;
        let public = PublicKey::from(public_bytes);
        let shared = secret.diffie_hellman(&public);
        Ok(shared.as_bytes().to_vec())
    }

    /// Perform hybrid key agreement (Kyber + X25519)
    pub fn hybrid_key_agreement(
        &self,
        kyber_ss: &[u8],
        x25519_ss: &[u8],
    ) -> Result<HybridSharedSecret, Box<dyn std::error::Error>> {
        let mut combined = Vec::with_capacity(kyber_ss.len() + x25519_ss.len());
        combined.extend_from_slice(kyber_ss);
        combined.extend_from_slice(x25519_ss);
        let hk = Hkdf::<Sha3_256>::new(None, &combined);
        let mut session_key = [0u8; 32];
        hk.expand(b"qsafe-session-key", &mut session_key)
            .map_err(|e| format!("HKDF expand failure: {:?}", e))?;
        combined.zeroize(); // Wipe the combined shared secrets buffer
        let secret = HybridSharedSecret {
            kyber_ss: kyber_ss.to_vec(),
            x25519_ss: x25519_ss.to_vec(),
            session_key: session_key.to_vec(),
        };
        session_key.zeroize(); // Wipe the stack session key buffer
        Ok(secret)
    }

    /// Generate Ed25519 keypair for identity signatures
    pub fn generate_ed25519_keypair(&mut self) -> Result<KeyPair, Box<dyn std::error::Error>> {
        let mut csprng = rand::rngs::OsRng;
        let mut bytes = [0u8; 32];
        csprng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        bytes.zeroize(); // Wipe the csprng buffer
        let mut secret_bytes = signing_key.to_bytes();
        let keypair = KeyPair {
            public_key: signing_key.verifying_key().to_bytes().to_vec(),
            secret_key: secret_bytes.to_vec(),
        };
        secret_bytes.zeroize(); // Wipe stack buffer copy
        Ok(keypair)
    }

    /// Sign a message using Ed25519
    pub fn sign_ed25519(
        &self,
        secret_key: &[u8],
        message: &[u8],
    ) -> Result<QSafeSignature, Box<dyn std::error::Error>> {
        let mut secret_bytes = <[u8; 32]>::try_from(secret_key)?;
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        secret_bytes.zeroize(); // Clear the stack copy of seed bytes
        let signature = signing_key.sign(message);
        Ok(QSafeSignature {
            signature: signature.to_bytes().to_vec(),
            message: message.to_vec(),
        })
    }

    /// Verify an Ed25519 signature
    pub fn verify_ed25519(
        &self,
        public_key: &[u8],
        signature: &QSafeSignature,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let verifying_key = VerifyingKey::from_bytes(&public_key.try_into()?)?;
        let sig_bytes: [u8; 64] = signature
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid signature size")?;
        let sig = Signature::from_bytes(&sig_bytes);
        Ok(verifying_key.verify(&signature.message, &sig).is_ok())
    }

    /// Compute authenticated key confirmation MAC
    pub fn key_confirmation_mac(&self, key: &[u8], message: &[u8]) -> Vec<u8> {
        let key_hash = blake3::hash(key);
        blake3::keyed_hash(key_hash.as_bytes(), message)
            .as_bytes()
            .to_vec()
    }

    /// Generate a hash using Blake3
    pub fn hash(&self, data: &[u8]) -> Vec<u8> {
        blake3::hash(data).as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kyber_encapsulate_decapsulate_round_trip() {
        let mut engine = CryptoEngine::new();
        let kp = engine.generate_kyber_keypair().unwrap();

        let (encapsulated_ss, ciphertext) = engine.kyber_encapsulate(&kp.public_key).unwrap();
        let decapsulated_ss = engine
            .kyber_decapsulate(&kp.secret_key, &ciphertext)
            .unwrap();

        assert_eq!(encapsulated_ss, decapsulated_ss);
    }

    #[test]
    fn x25519_shared_secret_matches_on_both_sides() {
        let mut engine = CryptoEngine::new();
        let alice = engine.generate_x25519_keypair().unwrap();
        let bob = engine.generate_x25519_keypair().unwrap();

        let alice_view = engine
            .x25519_shared_secret(&alice.secret_key, &bob.public_key)
            .unwrap();
        let bob_view = engine
            .x25519_shared_secret(&bob.secret_key, &alice.public_key)
            .unwrap();

        assert_eq!(alice_view, bob_view);
    }

    #[test]
    fn hybrid_key_agreement_derives_matching_session_keys() {
        let mut engine = CryptoEngine::new();

        // Simulate both sides landing on the same two shared secrets (as a
        // real handshake would after the Kyber and X25519 exchanges).
        let alice_kyber = engine.generate_kyber_keypair().unwrap();
        let (kyber_ss, kyber_ct) = engine.kyber_encapsulate(&alice_kyber.public_key).unwrap();
        let kyber_ss_alice = engine
            .kyber_decapsulate(&alice_kyber.secret_key, &kyber_ct)
            .unwrap();

        let alice_x25519 = engine.generate_x25519_keypair().unwrap();
        let bob_x25519 = engine.generate_x25519_keypair().unwrap();
        let x25519_ss_alice = engine
            .x25519_shared_secret(&alice_x25519.secret_key, &bob_x25519.public_key)
            .unwrap();
        let x25519_ss_bob = engine
            .x25519_shared_secret(&bob_x25519.secret_key, &alice_x25519.public_key)
            .unwrap();

        let alice_session = engine
            .hybrid_key_agreement(&kyber_ss_alice, &x25519_ss_alice)
            .unwrap();
        let bob_session = engine
            .hybrid_key_agreement(&kyber_ss, &x25519_ss_bob)
            .unwrap();

        assert_eq!(alice_session.session_key, bob_session.session_key);
        assert_eq!(alice_session.session_key.len(), 32);
    }

    #[test]
    fn hybrid_key_agreement_differs_for_different_inputs() {
        let mut engine = CryptoEngine::new();
        let kp1 = engine.generate_kyber_keypair().unwrap();
        let kp2 = engine.generate_kyber_keypair().unwrap();
        let (ss1, _) = engine.kyber_encapsulate(&kp1.public_key).unwrap();
        let (ss2, _) = engine.kyber_encapsulate(&kp2.public_key).unwrap();

        let secret1 = engine.hybrid_key_agreement(&ss1, &[0u8; 32]).unwrap();
        let secret2 = engine.hybrid_key_agreement(&ss2, &[0u8; 32]).unwrap();

        assert_ne!(secret1.session_key, secret2.session_key);
    }

    #[test]
    fn ed25519_sign_and_verify_round_trip() {
        let mut engine = CryptoEngine::new();
        let kp = engine.generate_ed25519_keypair().unwrap();
        let message = b"quantum-safe handshake init";

        let signature = engine.sign_ed25519(&kp.secret_key, message).unwrap();
        assert!(engine.verify_ed25519(&kp.public_key, &signature).unwrap());
    }

    #[test]
    fn ed25519_verify_rejects_tampered_message() {
        let mut engine = CryptoEngine::new();
        let kp = engine.generate_ed25519_keypair().unwrap();
        let mut signature = engine
            .sign_ed25519(&kp.secret_key, b"original message")
            .unwrap();
        signature.message = b"tampered message".to_vec();

        assert!(!engine.verify_ed25519(&kp.public_key, &signature).unwrap());
    }

    #[test]
    fn ed25519_verify_rejects_wrong_key() {
        let mut engine = CryptoEngine::new();
        let signer = engine.generate_ed25519_keypair().unwrap();
        let impostor = engine.generate_ed25519_keypair().unwrap();
        let signature = engine
            .sign_ed25519(&signer.secret_key, b"who signed this?")
            .unwrap();

        assert!(!engine
            .verify_ed25519(&impostor.public_key, &signature)
            .unwrap());
    }

    #[test]
    fn key_confirmation_mac_matches_for_same_key_and_differs_for_different_keys() {
        let engine = CryptoEngine::new();
        let key_a = [1u8; 32];
        let key_b = [2u8; 32];

        let mac_a1 = engine.key_confirmation_mac(&key_a, b"bob-confirmation");
        let mac_a2 = engine.key_confirmation_mac(&key_a, b"bob-confirmation");
        let mac_b = engine.key_confirmation_mac(&key_b, b"bob-confirmation");

        assert_eq!(mac_a1, mac_a2);
        assert_ne!(mac_a1, mac_b);
    }
}
