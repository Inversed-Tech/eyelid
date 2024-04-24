//! The underlying integer field.
//!
//! Outside this module, use [`fq::Coeff`](Coeff) instead of `fq79` or `fq_tiny`.
//! This automatically enables CI tests on both fields.

use crate::primitives::poly::fq::fq79::Fq79;
use ark_ff::UniformRand;
use ark_ff::Zero;
use lazy_static::lazy_static;
use rand::rngs::ThreadRng;

mod fq79;
mod fq_tiny;

#[cfg(not(tiny_poly))]
pub use fq79::Coeff;

#[cfg(not(tiny_poly))]
pub fn rand_coeff(rng: &mut ThreadRng) -> Fq79 {
    Fq79::rand(rng)
}

// Temporarily switch to this tiny field to make test errors easier to debug:
// ```no_run
// RUSTFLAGS="--cfg tiny_poly" cargo test
// RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
// ```
#[cfg(tiny_poly)]
pub use fq_tiny::Coeff;

#[cfg(tiny_poly)]
pub fn rand_coeff(rng: &mut ThreadRng) -> Fq4 {
    let res = Fq4::rand(rng);
    res
}

lazy_static! {
    /// The zero coefficient as a static constant value.
    ///
    /// # Usage
    ///
    /// Return `&super::fq::COEFF_ZERO` from a function that returns a reference to `Coeff::zero()`.
    ///
    /// Only use this constant when you need a long-lived reference to a zero coefficient value.
    /// The compiler will tell you, with errors like:
    /// > cannot return reference to a temporary value
    /// > returns a reference to data owned by the current function
    ///
    /// Typically, `Coeff::zero()` is more readable and efficient.
    pub static ref COEFF_ZERO: Coeff = {
        Coeff::zero()
    };
}
