//! Middle-resolution parameters in 2^66.
//!
//! These are the parameters for middle resolution, according to the Inversed Tech report.
//! t = 2ˆ12, q = 2ˆ66

use ark_ff::{Fp128, MontBackend, MontConfig};

/// The configuration of the modular field used for polynomial coefficients.
//
// Sage commands:
// random_prime(2**66)
// 21462786190088845153
// ff = GF(21462786190088845153)
// ff.multiplicative_generator()
// 5
//
// We could also consider generating primes dynamically, but this could impact performance.
#[derive(MontConfig)]
#[modulus = "21462786190088845153"]
#[generator = "5"]
pub struct Fq66Config;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
pub type Fq66 = Fp128<MontBackend<Fq66Config, 2>>;
