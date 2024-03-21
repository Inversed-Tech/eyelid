//! Fq79 implemented using two [`u64`]s.

use ark_ff::{Fp128, MontBackend, MontConfig};

/// The configuration of the modular field used for polynomial coefficients.
#[derive(MontConfig)]
#[modulus = "93309596432438992665667"]
#[generator = "5"]
pub struct Fq79Config;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
/// t = 2ˆ15, q = 2ˆ79, N = 2048
//
// Sage commands:
// random_prime(2**79)
// 93309596432438992665667
// ff = GF(93309596432438992665667)
// ff.multiplicative_generator()
// 5
//
// We could also consider generating primes dynamically, but this could impact performance.
pub type Fq79 = Fp128<MontBackend<Fq79Config, 2>>;
