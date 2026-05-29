//! Cryptographic primitives for Q-Safe

use blake3;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use orion::aead::{open, seal, SecretKey as AeadSecretKey};
use pqcrypto_kyber::kyber768::{decapsulate, encapsulate, keypair};
use pqcrypto_traits::kem::{
    Ciphertext as KyberCiphertext, PublicKey as KyberPublicKey, SecretKey as KyberSecretKey,
    SharedSecret as KyberSharedSecret,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha3::Sha3_256;
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
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

pub struct CryptoEngine {
    rng: rand::rngs::OsRng,
}

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
        let secret = StaticSecret::random_from_rng(&mut self.rng);
        let public = PublicKey::from(&secret);
        Ok(KeyPair {
            public_key: public.to_bytes().to_vec(),
            secret_key: secret.to_bytes().to_vec(),
        })
    }

    /// Compute X25519 shared secret
    pub fn x25519_shared_secret(
        &self,
        secret_key: &[u8],
        public_key: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let secret_bytes = <[u8; 32]>::try_from(secret_key)?;
        let secret = StaticSecret::from(secret_bytes);
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

        Ok(HybridSharedSecret {
            kyber_ss: kyber_ss.to_vec(),
            x25519_ss: x25519_ss.to_vec(),
            session_key: session_key.to_vec(),
        })
    }

    /// Generate Ed25519 keypair for identity signatures
    pub fn generate_ed25519_keypair(&mut self) -> Result<KeyPair, Box<dyn std::error::Error>> {
        let mut csprng = rand::rngs::OsRng;
        let mut bytes = [0u8; 32];
        csprng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        Ok(KeyPair {
            public_key: signing_key.verifying_key().to_bytes().to_vec(),
            secret_key: signing_key.to_bytes().to_vec(),
        })
    }

    /// Sign a message using Ed25519
    pub fn sign_ed25519(
        &self,
        secret_key: &[u8],
        message: &[u8],
    ) -> Result<QSafeSignature, Box<dyn std::error::Error>> {
        let signing_key = SigningKey::from_bytes(&secret_key.try_into()?);
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

    /// Encrypt a message using ChaCha20-Poly1305 AEAD
    pub fn encrypt_aead(
        &mut self,
        key: &[u8],
        plaintext: &[u8],
        _aad: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        let secret_key = AeadSecretKey::from_slice(key)?;
        let sealed = seal(&secret_key, plaintext)?;
        if sealed.len() < 28 {
            return Err("Invalid sealed data length".into());
        }
        let nonce = sealed[0..12].to_vec();
        let ciphertext = sealed[12..].to_vec();
        Ok((ciphertext, nonce))
    }

    /// Decrypt a message using ChaCha20-Poly1305 AEAD
    pub fn decrypt_aead(
        &self,
        key: &[u8],
        ciphertext: &[u8],
        nonce: &[u8],
        _aad: Option<&[u8]>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let secret_key = AeadSecretKey::from_slice(key)?;
        let mut full = nonce.to_vec();
        full.extend_from_slice(ciphertext);
        let plaintext = open(&secret_key, &full)?;
        Ok(plaintext)
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

    // Legacy methods for compatibility (deprecated)
    pub fn generate_pq_keypair(&mut self) -> Result<KeyPair, Box<dyn std::error::Error>> {
        self.generate_kyber_keypair()
    }

    pub fn encapsulate(
        &mut self,
        public_key: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        self.kyber_encapsulate(public_key)
    }

    pub fn decapsulate(
        &mut self,
        secret_key: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.kyber_decapsulate(secret_key, ciphertext)
    }

    pub fn sign(
        &mut self,
        secret_key: &[u8],
        message: &[u8],
    ) -> Result<QSafeSignature, Box<dyn std::error::Error>> {
        self.sign_ed25519(secret_key, message)
    }

    pub fn verify(
        &mut self,
        public_key: &[u8],
        signature: &QSafeSignature,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        self.verify_ed25519(public_key, signature)
    }

    pub fn encrypt(
        &mut self,
        key: &[u8],
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        let (ciphertext, nonce) = self.encrypt_aead(key, plaintext, None)?;
        Ok((ciphertext, nonce, vec![]))
    }

    pub fn decrypt(
        &mut self,
        key: &[u8],
        ciphertext: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.decrypt_aead(key, ciphertext, nonce, None)
    }
}
