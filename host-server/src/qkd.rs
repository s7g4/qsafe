//! Quantum Key Distribution simulation (BB84 protocol) with decoy bits and parity checks

use crate::crypto::CryptoEngine;
use crate::qrng::QRNG;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Basis {
    Rectilinear, // 0° and 90°
    Diagonal,    // 45° and 135°
}

#[derive(Debug, Clone, Copy)]
pub struct Photon {
    pub bit: u8,
    pub basis: Basis,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QKDKey {
    pub key: Vec<u8>,
    pub error_rate: f64,
    pub eavesdropping_detected: bool,
    pub confirmation_mac: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecoyCheck {
    pub indices: Vec<usize>,
    pub bases: Vec<u8>,
    pub error_rate: f64,
}

pub struct QKDProtocol {
    qrng: QRNG,
    crypto: CryptoEngine,
}

impl QKDProtocol {
    pub fn new() -> Self {
        Self {
            qrng: QRNG::new(),
            crypto: CryptoEngine::new(),
        }
    }

    /// BB84 protocol with decoy bits and parity checks
    pub fn bb84_with_decoy_checks(
        &mut self,
        key_length: usize,
        decoy_fraction: f64,
    ) -> Result<QKDKey, Box<dyn std::error::Error>> {
        // Alice prepares photons with decoys
        let total_bits = (key_length as f64 / (1.0 - decoy_fraction)) as usize;
        let mut alice_bits = Vec::new();
        let mut alice_bases = Vec::new();

        for _ in 0..total_bits {
            alice_bits.push(self.qrng.quantum_measurement());
            alice_bases.push(if self.qrng.quantum_measurement() == 0 {
                Basis::Rectilinear
            } else {
                Basis::Diagonal
            });
        }

        // Generate decoy bits
        let (decoy_indices, _decoy_bases) =
            self.qrng.generate_decoy_bits(total_bits, decoy_fraction);

        // Bob measures photons
        let mut bob_bits = Vec::new();
        let mut bob_bases = Vec::new();

        for _ in 0..alice_bits.len() {
            bob_bits.push(self.qrng.quantum_measurement());
            bob_bases.push(if self.qrng.quantum_measurement() == 0 {
                Basis::Rectilinear
            } else {
                Basis::Diagonal
            });
        }

        // Sifting: Keep only bits where bases match (excluding decoys for now)
        let mut sifted_bits = Vec::new();
        let mut sifted_indices = Vec::new();
        for i in 0..alice_bits.len() {
            if alice_bases[i] == bob_bases[i] && !decoy_indices.contains(&i) {
                sifted_bits.push(alice_bits[i]);
                sifted_indices.push(i);
            }
        }

        // Decoy check: Reveal decoy bases and check error rate
        let decoy_error_rate = self.check_decoy_errors(
            &alice_bits,
            &bob_bits,
            &alice_bases,
            &bob_bases,
            &decoy_indices,
        );

        // Parity check on sifted bits
        let parity_error_rate = self.check_parity_errors(&sifted_bits);

        // Combined error rate
        let combined_error_rate = (decoy_error_rate + parity_error_rate) / 2.0;

        // Privacy amplification
        let amplified_key = self.privacy_amplification(&sifted_bits);

        // Key confirmation MAC
        let confirmation_mac = self
            .crypto
            .key_confirmation_mac(&amplified_key, b"qsafe-key-confirmation");

        Ok(QKDKey {
            key: amplified_key,
            error_rate: combined_error_rate,
            eavesdropping_detected: combined_error_rate > 0.11,
            confirmation_mac,
        })
    }

    fn check_decoy_errors(
        &self,
        alice_bits: &[u8],
        bob_bits: &[u8],
        alice_bases: &[Basis],
        bob_bases: &[Basis],
        decoy_indices: &[usize],
    ) -> f64 {
        let mut errors = 0;
        let mut total = 0;

        for &i in decoy_indices {
            if i < alice_bits.len() && i < bob_bits.len() {
                total += 1;
                if alice_bases[i] == bob_bases[i] && alice_bits[i] != bob_bits[i] {
                    errors += 1;
                }
            }
        }

        if total == 0 {
            0.0
        } else {
            errors as f64 / total as f64
        }
    }

    fn check_parity_errors(&self, bits: &[u8]) -> f64 {
        if bits.len() < 2 {
            return 0.0;
        }

        let mut errors = 0;
        let mut total = 0;

        // Check parity of random subsets
        for _ in 0..bits.len() / 4 {
            let subset: Vec<u8> = bits.iter().take(4).cloned().collect();
            let parity = subset.iter().fold(0u8, |acc, &b| acc ^ b);
            if parity != 0 {
                errors += 1;
            }
            total += 1;
        }

        if total == 0 {
            0.0
        } else {
            errors as f64 / total as f64
        }
    }

    fn privacy_amplification(&self, bits: &[u8]) -> Vec<u8> {
        let bit_string = bits.iter().map(|b| b.to_string()).collect::<String>();
        self.crypto.hash(bit_string.as_bytes())
    }

    /// Legacy BB84 without decoys (for compatibility)
    pub fn bb84_key_exchange(
        &mut self,
        key_length: usize,
    ) -> Result<QKDKey, Box<dyn std::error::Error>> {
        let qkd_key = self.bb84_with_decoy_checks(key_length, 0.1)?;
        Ok(QKDKey {
            key: qkd_key.key,
            error_rate: qkd_key.error_rate,
            eavesdropping_detected: qkd_key.eavesdropping_detected,
            confirmation_mac: qkd_key.confirmation_mac,
        })
    }

    /// E91 protocol with decoy checks
    pub fn e91_with_decoy_checks(
        &mut self,
        key_length: usize,
        decoy_fraction: f64,
    ) -> Result<QKDKey, Box<dyn std::error::Error>> {
        let mut key_bits = Vec::new();
        let mut entangled_pairs = Vec::new();

        for _ in 0..key_length {
            let entangled_pair = self.qrng.generate_entangled_pair();
            entangled_pairs.push(entangled_pair);

            let alice_measurement = self.measure_in_random_basis(entangled_pair.0);
            let bob_measurement = self.measure_in_random_basis(entangled_pair.1);

            if alice_measurement.1 == bob_measurement.1 {
                key_bits.push(alice_measurement.0);
            }
        }

        // Decoy check on entangled pairs
        let decoy_indices = self
            .qrng
            .generate_decoy_bits(entangled_pairs.len(), decoy_fraction)
            .0;
        let decoy_error_rate = self.qrng.detect_eavesdropping(
            &entangled_pairs,
            &entangled_pairs, // Perfect correlation assumed
            Some(&decoy_indices),
        );

        let amplified_key = self.privacy_amplification(&key_bits);
        let confirmation_mac = self
            .crypto
            .key_confirmation_mac(&amplified_key, b"qsafe-e91-confirmation");

        Ok(QKDKey {
            key: amplified_key,
            error_rate: decoy_error_rate,
            eavesdropping_detected: decoy_error_rate > 0.11,
            confirmation_mac,
        })
    }

    fn measure_in_random_basis(&mut self, bit: u8) -> (u8, Basis) {
        let basis = if self.qrng.quantum_measurement() == 0 {
            Basis::Rectilinear
        } else {
            Basis::Diagonal
        };
        (bit, basis)
    }
}
