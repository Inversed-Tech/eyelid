//! "BigNum" for Full-resolution parameters in 2^79.

use ark_ff::{Fp192, MontBackend, MontConfig};

/// The configuration of the modular field used for polynomial coefficients.
//
// Sage commands:
// size_q = 79
// size_n = 11
// size = 2*size_q + size_n + 1
// q = random_prime(2**(2*79 + 11 + 1))
// ff = GF(q)
// ff.multiplicative_generator()
// 10
#[derive(MontConfig)]
#[modulus = "292599365471450699889605790713940231423010469055097"]
#[generator = "10"]
pub struct Fq79bnConfig;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
pub type Fq79bn = Fp192<MontBackend<Fq79bnConfig, 3>>;
