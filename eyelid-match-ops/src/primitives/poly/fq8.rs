//! Tiny test-only parameters in 2^8.
//!
//! These test parameters are specifically chosen to make failing tests easy to read and diagnose.
//! q = 2ˆ8, N = 4

use ark_ff::{Fp64, MontBackend, MontConfig};

/// The maximum exponent in the test-only polynomial.
pub const MAX_POLY_DEGREE: usize = 4;

/// The modular field used for test polynomial coefficients, with precomputed primes and generators.
pub type Coeff = Fq8;

/// The configuration of the test-only modular field, used for polynomial coefficients.
//
// Sage commands:
// random_prime(2**8)
// 239
// ff = GF(239)
// ff.multiplicative_generator()
// 7
//
// We could also consider generating primes dynamically, but this could impact performance.
#[derive(MontConfig)]
#[modulus = "239"]
#[generator = "7"]
pub struct Fq8Config;

/// The modular field used for test polynomial coefficients, with precomputed primes and generators.
pub type Fq8 = Fp64<MontBackend<Fq8Config, 1>>;
