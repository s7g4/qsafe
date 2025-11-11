//! Quantum Key Distribution simulation (BB84 protocol)

use crate::qrng::QRNG;
use crate::crypto::CryptoEngine;
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
}

pub struct QKDProtocol {
    qrng: QRNG,
    crypto: CryptoEngine,
}

impl QKDProtocol {
    pub fn new(seed: u64) -> Self {
        Self {
            qrng: QRNG::new(seed),
            crypto: CryptoEngine::new(),
        }
    }

    /// Simulate BB84 protocol key exchange
    pub fn bb84_key_exchange(&mut self, key_length: usize) -> Result<QKDKey, Box<dyn std::error::Error>> {
        // Alice prepares photons
        let mut alice_bits = Vec::new();
        let mut alice_bases = Vec::new();

        for _ in 0..key_length * 4 { // Generate more bits for sifting
            alice_bits.push(self.qrng.quantum_measurement());
            alice_bases.push(if self.qrng.quantum_measurement() == 0 { Basis::Rectilinear } else { Basis::Diagonal });
        }

        // Bob measures photons
        let mut bob_bits = Vec::new();
        let mut bob_bases = Vec::new();

        for _ in 0..alice_bits.len() {
            bob_bits.push(self.qrng.quantum_measurement());
            bob_bases.push(if self.qrng.quantum_measurement() == 0 { Basis::Rectilinear } else { Basis::Diagonal });
        }

        // Sifting: Keep only bits where bases match
        let mut sifted_bits = Vec::new();
        for i in 0..alice_bits.len() {
            if alice_bases[i] == bob_bases[i] {
                sifted_bits.push(alice_bits[i]);
            }
        }

        // Error estimation (simulate eavesdropping)
        let error_rate = self.estimate_error_rate(&alice_bits, &bob_bits, &alice_bases, &bob_bases);

        // Privacy amplification (simplified)
        let amplified_key = self.privacy_amplification(&sifted_bits);

        Ok(QKDKey {
            key: amplified_key,
            error_rate,
            eavesdropping_detected: error_rate > 0.11, // Threshold for eavesdropping detection
        })
    }

    fn estimate_error_rate(&self, alice_bits: &[u8], bob_bits: &[u8], alice_bases: &[Basis], bob_bases: &[Basis]) -> f64 {
        let mut errors = 0;
        let mut total = 0;

        for i in 0..alice_bits.len().min(bob_bits.len()) {
            if alice_bases[i] == bob_bases[i] {
                total += 1;
                if alice_bits[i] != bob_bits[i] {
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

    fn privacy_amplification(&self, bits: &[u8]) -> Vec<u8> {
        // Simplified privacy amplification using hash
        let bit_string = bits.iter().map(|b| b.to_string()).collect::<String>();
        self.crypto.hash(bit_string.as_bytes())
    }

    /// Simulate E91 protocol (entanglement-based QKD)
    pub fn e91_key_exchange(&mut self, key_length: usize) -> Result<QKDKey, Box<dyn std::error::Error>> {
        let mut key_bits = Vec::new();

        for _ in 0..key_length {
            // Generate entangled pair
            let entangled_pair = self.qrng.generate_entangled_pair();

            // Alice and Bob measure in random bases
            let alice_measurement = self.measure_in_random_basis(entangled_pair.0);
            let bob_measurement = self.measure_in_random_basis(entangled_pair.1);

            // If they chose the same basis, the bit is correlated
            if alice_measurement.1 == bob_measurement.1 {
                key_bits.push(alice_measurement.0);
            }
        }

        // Error estimation
        let error_rate = self.qrng.detect_eavesdropping(
            &key_bits.iter().map(|&b| (b, b)).collect::<Vec<_>>(),
            &key_bits.iter().map(|&b| (b, b)).collect::<Vec<_>>(),
        );

        let amplified_key = self.privacy_amplification(&key_bits);

        Ok(QKDKey {
            key: amplified_key,
            error_rate,
            eavesdropping_detected: error_rate > 0.11,
        })
    }

    fn measure_in_random_basis(&mut self, bit: u8) -> (u8, Basis) {
        let basis = if self.qrng.quantum_measurement() == 0 { Basis::Rectilinear } else { Basis::Diagonal };
        (bit, basis)
    }
}
