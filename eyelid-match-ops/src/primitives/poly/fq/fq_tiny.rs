//! Tiny test-only parameters in 2^4.
//!
//! These test parameters are specifically chosen to make failing tests easy to read and diagnose.
//! q = 2ˆ4

use ark_ff::{Fp64, MontBackend, MontConfig};

/// The configuration of the test-only modular field, used for polynomial coefficients.
///
/// Deliberately set to extremely small values, so that random polynomials are likely to have zeroes, ones, and minus ones.
//
// Sage commands, results from <https://sagecell.sagemath.org/>:
// random_prime(2**4)
// 7
// ff = GF(7)
// ff.multiplicative_generator()
// 3
//
// We could also consider generating primes dynamically, but this could impact performance.
#[derive(MontConfig)]
#[modulus = "7"]
#[generator = "3"]
pub struct Fq4Config;

/// The modular field used for test polynomial coefficients, with precomputed primes and generators.
pub type Fq4 = Fp64<MontBackend<Fq4Config, 1>>;
