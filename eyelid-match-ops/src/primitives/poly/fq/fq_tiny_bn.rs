//! "BigNum" for Full-resolution parameters in 2^4.

use ark_ff::{Fp64, MontBackend, MontConfig};

/// The configuration of the test-only modular field, used for polynomial coefficients (bn).
///
/// Deliberately set to extremely small values, so that random polynomials are likely to have zeroes, ones, and minus ones.
// random_prime(2**13)
// 5399
// ff = GF(5399)
// ff.multiplicative_generator()
// 7
//
// We could also consider generating primes dynamically, but this could impact performance.
#[derive(MontConfig)]
#[modulus = "5399"]
#[generator = "7"]
pub struct Fq4Config;

/// The modular field used for test polynomial coefficients, with precomputed primes and generators.
pub type Fq4 = Fp64<MontBackend<Fq4Config, 1>>;
