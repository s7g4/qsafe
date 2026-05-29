//! Hybrid handshake protocol for Q-Safe (Kyber + X25519 + decoy checks)

use crate::crypto::{CryptoEngine, QSafeSignature};
use crate::qkd::{DecoyCheck, QKDProtocol};
use crate::qrng::QRNG;
use blake3;
use serde::{Deserialize, Serialize};
use std::error::Error;
use zeroize::Zeroize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandshakeInit {
    pub kyber_pk: Vec<u8>,
    pub x25519_pub: Vec<u8>,
    pub decoy_commit: Vec<u8>,        // Blake3 hash of decoy bits/bases
    pub identity_sig: QSafeSignature, // Signed init message
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandshakeResponse {
    pub kyber_ct: Vec<u8>,
    pub x25519_pub: Vec<u8>,
    pub decoy_reveal: DecoyCheck,
    pub confirmation_hash: Vec<u8>, // Keyed Blake3 of session_key
    pub identity_sig: QSafeSignature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeResult {
    pub session_key: Vec<u8>,
    pub error_rate: f64,
    pub eavesdropping_detected: bool,
    pub confirmed: bool,
}

pub struct Handshake {
    crypto: CryptoEngine,
    qkd: QKDProtocol,
    qrng: QRNG,
}

#[allow(clippy::new_without_default)]
impl Handshake {
    pub fn new() -> Self {
        Self {
            crypto: CryptoEngine::new(),
            qkd: QKDProtocol::new(),
            qrng: QRNG::new(),
        }
    }

    /// Alice initiates handshake
    pub fn alice_initiate(
        &mut self,
        key_length: usize,
        decoy_fraction: f64,
    ) -> Result<(HandshakeInit, Vec<u8>), Box<dyn Error>> {
        // Generate ephemeral keys
        let kyber_kp = self.crypto.generate_kyber_keypair()?;
        let x25519_kp = self.crypto.generate_x25519_keypair()?;

        // Generate decoy bits for check
        let (decoy_indices, decoy_bases) = self
            .qrng
            .generate_decoy_bits(key_length * 2, decoy_fraction);
        let decoy_data = bincode::serialize(&(decoy_indices, &decoy_bases[..]))?;
        let decoy_commit = blake3::hash(&decoy_data).as_bytes().to_vec();

        // Sign init
        let init_msg = bincode::serialize(&(
            kyber_kp.public_key.clone(),
            x25519_kp.public_key.clone(),
            decoy_commit.clone(),
        ))?;
        let identity_kp = self.crypto.generate_ed25519_keypair()?; // Assume persistent identity
        let identity_sig = self
            .crypto
            .sign_ed25519(&identity_kp.secret_key, &init_msg)?;

        let init = HandshakeInit {
            kyber_pk: kyber_kp.public_key,
            x25519_pub: x25519_kp.public_key,
            decoy_commit,
            identity_sig,
        };

        // Store private keys for later (in real impl, use secure storage)
        let private_data = bincode::serialize(&(
            kyber_kp.secret_key,
            x25519_kp.secret_key,
            identity_kp.secret_key,
            decoy_data,
        ))?;

        Ok((init, private_data))
    }

    /// Bob responds to init
    pub fn bob_respond(
        &mut self,
        init: &HandshakeInit,
        _private_alice_data: Option<Vec<u8>>,
    ) -> Result<(HandshakeResponse, Vec<u8>), Box<dyn Error>> {
        // Generate Bob's ephemeral keys
        let kyber_kp_bob = self.crypto.generate_kyber_keypair()?;
        let x25519_kp_bob = self.crypto.generate_x25519_keypair()?;

        // Encapsulate with Alice's Kyber PK
        let (kyber_ss, kyber_ct) = self.crypto.kyber_encapsulate(&init.kyber_pk)?;

        // Compute X25519 shared (Bob's secret * Alice's pub)
        let x25519_ss = self
            .crypto
            .x25519_shared_secret(&x25519_kp_bob.secret_key, &init.x25519_pub)?;

        // Derive session key
        let hybrid_ss = self.crypto.hybrid_key_agreement(&kyber_ss, &x25519_ss)?;
        let session_key = hybrid_ss.session_key.clone();

        // Decoy reveal and check (simulate channel)
        let decoy_check = self.perform_decoy_check(256, 0.2); // Placeholder

        // Key confirmation hash
        let confirmation_hash = self
            .crypto
            .key_confirmation_mac(&session_key, b"bob-confirmation");

        // Sign response
        let resp_msg = bincode::serialize(&(
            kyber_ct.clone(),
            x25519_kp_bob.public_key.clone(),
            decoy_check.clone(),
            confirmation_hash.clone(),
        ))?;
        let identity_kp_bob = self.crypto.generate_ed25519_keypair()?;
        let identity_sig = self
            .crypto
            .sign_ed25519(&identity_kp_bob.secret_key, &resp_msg)?;

        let response = HandshakeResponse {
            kyber_ct,
            x25519_pub: x25519_kp_bob.public_key,
            decoy_reveal: decoy_check,
            confirmation_hash,
            identity_sig,
        };

        // Store Bob's private
        let private_data = bincode::serialize(&(
            kyber_kp_bob.secret_key,
            x25519_kp_bob.secret_key,
            identity_kp_bob.secret_key,
            session_key,
        ))?;

        Ok((response, private_data))
    }

    /// Alice finalizes handshake
    pub fn alice_finalize(
        &mut self,
        init: &HandshakeInit,
        response: &HandshakeResponse,
        private_data: Vec<u8>,
    ) -> Result<HandshakeResult, Box<dyn Error>> {
        let (mut kyber_sk, mut x25519_sk, _, decoy_data): (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) =
            bincode::deserialize(&private_data)?;

        // Decapsulate Kyber
        let kyber_ss = self
            .crypto
            .kyber_decapsulate(&kyber_sk, &response.kyber_ct)?;

        // Compute X25519 shared (Alice's secret * Bob's pub)
        let x25519_ss = self
            .crypto
            .x25519_shared_secret(&x25519_sk, &response.x25519_pub)?;

        // Derive session key
        let hybrid_ss = self.crypto.hybrid_key_agreement(&kyber_ss, &x25519_ss)?;
        let mut session_key = hybrid_ss.session_key;

        // Verify decoy commit
        let (decoy_indices, decoy_bases): (Vec<usize>, Vec<u8>) =
            bincode::deserialize(&decoy_data)?;
        let decoy_check_data = bincode::serialize(&(decoy_indices, &decoy_bases[..]))?;
        let computed_commit = blake3::hash(&decoy_check_data).as_bytes().to_vec();
        if computed_commit != init.decoy_commit {
            // Assume init from context
            return Err("Decoy commit mismatch".into());
        }

        // Check error rate from decoy reveal
        let error_rate = response.decoy_reveal.error_rate;
        let eavesdropping_detected = error_rate > 0.11;

        if eavesdropping_detected {
            session_key.zeroize();
            return Err("Eavesdropping detected".into());
        }

        // Verify confirmation hash (recompute expected)
        let expected_confirm = self
            .crypto
            .key_confirmation_mac(&session_key, b"alice-confirmation");
        let confirmed = expected_confirm == response.confirmation_hash;

        // Zeroize secrets
        kyber_sk.zeroize();
        x25519_sk.zeroize();

        Ok(HandshakeResult {
            session_key,
            error_rate,
            eavesdropping_detected: false,
            confirmed,
        })
    }

    /// Bob verifies confirmation from Alice (symmetric)
    pub fn bob_verify_confirmation(&self, alice_confirm: &[u8], session_key: &[u8]) -> bool {
        let expected = self
            .crypto
            .key_confirmation_mac(session_key, b"alice-confirmation");
        expected == alice_confirm
    }

    fn perform_decoy_check(&mut self, key_length: usize, decoy_fraction: f64) -> DecoyCheck {
        // Simulate decoy check using QKD
        let qkd_key = self
            .qkd
            .bb84_with_decoy_checks(key_length, decoy_fraction)
            .unwrap();
        DecoyCheck {
            indices: vec![], // Actual indices would be sent
            bases: vec![],
            error_rate: qkd_key.error_rate,
        }
    }
}
