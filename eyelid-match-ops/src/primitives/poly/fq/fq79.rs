//! Full-resolution parameters in 2^79.
//!
//! These are the parameters for full resolution, according to the Inversed Tech report.
//! t = 2ˆ15, q = 2ˆ79

use ark_ff::{Fp128, MontBackend, MontConfig};

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
/// TODO: delete this alias as part of cleanup.
pub type Coeff = Fq79;

/// The configuration of the modular field used for polynomial coefficients.
//
// Sage commands:
// random_prime(2**79)
// 93309596432438992665667
// ff = GF(93309596432438992665667)
// ff.multiplicative_generator()
// 5
//
// We could also consider generating primes dynamically, but this could impact performance.
#[derive(MontConfig)]
#[modulus = "93309596432438992665667"]
#[generator = "5"]
pub struct Fq79Config;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
pub type Fq79 = Fp128<MontBackend<Fq79Config, 2>>;
