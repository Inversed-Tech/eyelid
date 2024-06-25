//! "BigNum" for Full-resolution parameters in 2^123.

use ark_ff::{Fp320, MontBackend, MontConfig};

/// The configuration of the modular field used for polynomial coefficients.
//
// Sage commands:
// size_q = 123
// size_n = 11
// size_bn = 2*size_q + size_n + 1
// q = random_prime(2**(2*size_q + size_n + 1))
// ff = GF(q)
// ff.multiplicative_generator()
// 10
#[derive(MontConfig)]
#[modulus = "213557444820616770446469265530687396088842657026576979183513525061054745996489"]
#[generator = "3"]
pub struct Fq123bnConfig;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
pub type Fq123bn = Fp320<MontBackend<Fq123bnConfig, 5>>;
