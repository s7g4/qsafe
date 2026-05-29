//! Quantum Random Number Generation with secure entropy

use rand::RngCore;
use rand::rngs::OsRng;
use rand_pcg::Pcg64Mcg;
use rand::SeedableRng;
use rand::Rng;
use getrandom;

pub struct QRNG {
    secure_rng: OsRng,
    #[cfg(test)]
    sim_rng: Option<Pcg64Mcg>,
}

impl QRNG {
    /// Create a new QRNG instance with secure entropy
    pub fn new() -> Self {
        Self {
            secure_rng: OsRng,
            #[cfg(test)]
            sim_rng: None,
        }
    }

    /// Create a new QRNG instance with deterministic simulation for tests
    #[cfg(test)]
    pub fn new_sim(seed: u64) -> Self {
        Self {
            secure_rng: OsRng,
            sim_rng: Some(Pcg64Mcg::seed_from_u64(seed)),
        }
    }

    /// Generate fresh random bytes using high-quality OS entropy
    pub fn fresh_random_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        #[cfg(not(test))]
        self.secure_rng.fill_bytes(&mut bytes);
        #[cfg(test)]
        if let Some(ref mut rng) = self.sim_rng {
            rng.fill_bytes(&mut bytes);
        } else {
            self.secure_rng.fill_bytes(&mut bytes);
        }
        bytes
    }

    /// Generate a random byte
    pub fn random_byte(&mut self) -> u8 {
        let mut byte = [0u8; 1];
        #[cfg(not(test))]
        self.secure_rng.fill_bytes(&mut byte);
        #[cfg(test)]
        if let Some(ref mut rng) = self.sim_rng {
            rng.fill_bytes(&mut byte);
        } else {
            self.secure_rng.fill_bytes(&mut byte);
        }
        byte[0]
    }

    /// Generate a random vector of bytes
    pub fn random_bytes(&mut self, len: usize) -> Vec<u8> {
        self.fresh_random_bytes(len)
    }

    /// Simulate quantum measurement (random 0 or 1)
    pub fn quantum_measurement(&mut self) -> u8 {
        self.random_byte() % 2
    }

    /// Generate entangled pair simulation
    pub fn generate_entangled_pair(&mut self) -> (u8, u8) {
        let bit1 = self.quantum_measurement();
        let bit2 = bit1; // Perfect correlation for entangled pair
        (bit1, bit2)
    }

    /// Detect eavesdropping with decoy bits support
    pub fn detect_eavesdropping(&mut self, original_pairs: &[(u8, u8)], measured_pairs: &[(u8, u8)], decoy_indices: Option<&[usize]>) -> f64 {
        let pairs_to_check = if let Some(indices) = decoy_indices {
            indices.iter().filter_map(|&i| {
                if i < original_pairs.len() && i < measured_pairs.len() {
                    Some((original_pairs[i], measured_pairs[i]))
                } else {
                    None
                }
            }).collect::<Vec<_>>()
        } else {
            original_pairs.iter().zip(measured_pairs.iter()).map(|(&a, &b)| (a, b)).collect()
        };

        let mut errors = 0;
        for (orig, meas) in pairs_to_check {
            if orig.0 != meas.0 || orig.1 != meas.1 {
                errors += 1;
            }
        }
        if pairs_to_check.is_empty() {
            0.0
        } else {
            errors as f64 / pairs_to_check.len() as f64
        }
    }

    /// Generate decoy bits for BB84-style checks
    pub fn generate_decoy_bits(&mut self, total_bits: usize, decoy_fraction: f64) -> (Vec<usize>, Vec<u8>) {
        let decoy_count = (total_bits as f64 * decoy_fraction) as usize;
        let mut decoy_indices = Vec::with_capacity(decoy_count);
        let mut decoy_bases = Vec::with_capacity(decoy_count);

        // Randomly select decoy indices
        let mut available = (0..total_bits).collect::<Vec<_>>();
        for _ in 0..decoy_count {
            let idx = self.random_byte() as usize % available.len();
            decoy_indices.push(available.swap_remove(idx));
            decoy_bases.push(self.quantum_measurement()); // Random basis for decoy
        }

        (decoy_indices, decoy_bases)
    }
}
