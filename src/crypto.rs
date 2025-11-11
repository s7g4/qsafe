//! Cryptographic primitives for Q-Safe

use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use sha2::{Sha256, Digest};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signature {
    pub signature: Vec<u8>,
    pub message: Vec<u8>,
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

    /// Generate a post-quantum key pair using Kyber (simplified for demo)
    pub fn generate_pq_keypair(&mut self) -> Result<KeyPair, Box<dyn std::error::Error>> {
        // Simplified: generate random keys for demo
        let mut pk = vec![0u8; 32];
        let mut sk = vec![0u8; 32];
        self.rng.fill_bytes(&mut pk);
        self.rng.fill_bytes(&mut sk);
        Ok(KeyPair {
            public_key: pk,
            secret_key: sk,
        })
    }

    /// Perform post-quantum key encapsulation (simplified)
    pub fn encapsulate(&mut self, public_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        // Simplified: generate shared secret and ciphertext
        let mut ss = vec![0u8; 32];
        let mut ct = vec![0u8; 32];
        self.rng.fill_bytes(&mut ss);
        self.rng.fill_bytes(&mut ct);
        Ok((ss, ct))
    }

    /// Perform post-quantum key decapsulation (simplified)
    pub fn decapsulate(&mut self, secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Simplified: return a shared secret
        let mut ss = vec![0u8; 32];
        self.rng.fill_bytes(&mut ss);
        Ok(ss)
    }

    /// Sign a message using Dilithium (simplified)
    pub fn sign(&mut self, secret_key: &[u8], message: &[u8]) -> Result<Signature, Box<dyn std::error::Error>> {
        // Simplified: generate a random signature
        let mut sig = vec![0u8; 64];
        self.rng.fill_bytes(&mut sig);
        Ok(Signature {
            signature: sig,
            message: message.to_vec(),
        })
    }

    /// Verify a signature using Dilithium (simplified)
    pub fn verify(&mut self, public_key: &[u8], signature: &Signature) -> Result<bool, Box<dyn std::error::Error>> {
        // Simplified: always return true for demo
        Ok(true)
    }

    /// Encrypt a message using AES-GCM
    pub fn encrypt(&mut self, key: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {:?}", e))?;
        let mut nonce = [0u8; 12];
        self.rng.fill_bytes(&mut nonce);
        let nonce = Nonce::from_slice(&nonce);

        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {:?}", e))?;

        Ok((ciphertext, nonce.to_vec(), vec![])) // Tag is included in ciphertext for GCM
    }

    /// Decrypt a message using AES-GCM
    pub fn decrypt(&mut self, key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {:?}", e))?;
        let nonce = Nonce::from_slice(nonce);

        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {:?}", e))?;

        Ok(plaintext)
    }

    /// Generate a hash using SHA-256
    pub fn hash(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}
