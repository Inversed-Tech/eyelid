//! "BigNum" for Full-resolution parameters in 2^66.

use ark_ff::{Fp192, MontBackend, MontConfig};

/// The configuration of the modular field used for polynomial coefficients.
//
// Sage commands:
// size_q = 66
// size_n = 10
// size = 2*size_q + size_n + 1
// q = random_prime(2**(2*size_q + size_n + 1))
// ff = GF(q)
// ff.multiplicative_generator()
// 10
#[derive(MontConfig)]
#[modulus = "8810663000980779494481237054627323289751079"]
#[generator = "7"]
pub struct Fq66bnConfig;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
pub type Fq66bn = Fp192<MontBackend<Fq66bnConfig, 3>>;
