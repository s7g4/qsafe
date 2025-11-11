//! Quantum Random Number Generation simulation

use rand_pcg::Pcg64Mcg;
use rand::SeedableRng;
use rand::Rng;
use rand::RngCore;

pub struct QRNG {
    rng: Pcg64Mcg,
}

impl QRNG {
    /// Create a new QRNG instance with a seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Pcg64Mcg::seed_from_u64(seed),
        }
    }

    /// Generate a random byte
    pub fn random_byte(&mut self) -> u8 {
        self.rng.gen()
    }

    /// Generate a random vector of bytes
    pub fn random_bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.rng.gen()).collect()
    }

    /// Simulate quantum measurement (random 0 or 1)
    pub fn quantum_measurement(&mut self) -> u8 {
        self.rng.gen_range(0..2)
    }

    /// Generate entangled pair simulation
    pub fn generate_entangled_pair(&mut self) -> (u8, u8) {
        let bit1 = self.quantum_measurement();
        let bit2 = bit1; // Perfect correlation for entangled pair
        (bit1, bit2)
    }

    /// Simulate eavesdropping detection
    pub fn detect_eavesdropping(&mut self, original_pairs: &[(u8, u8)], measured_pairs: &[(u8, u8)]) -> f64 {
        let mut errors = 0;
        for (orig, meas) in original_pairs.iter().zip(measured_pairs.iter()) {
            if orig.0 != meas.0 || orig.1 != meas.1 {
                errors += 1;
            }
        }
        errors as f64 / original_pairs.len() as f64
    }
}
