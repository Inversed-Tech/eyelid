//! The underlying integer field.
//!
//! Outside this module, use [`PolyConf::Coeff`] instead of [`Fq79`] or [`FqTiny`].
//! This automatically enables CI tests on both fields.

pub use fq79::Fq79;
pub use fq_tiny::Fq4 as FqTiny;

use ark_ff::Zero;
use lazy_static::lazy_static;

// Doc links only
#[allow(unused_imports)]
use crate::primitives::poly::PolyConf;

mod fq79;
mod fq_tiny;

#[cfg(not(tiny_poly))]
pub use fq79::Coeff;

// Temporarily switch to this tiny field to make test errors easier to debug:
// ```no_run
// RUSTFLAGS="--cfg tiny_poly" cargo test
// RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
// ```
#[cfg(tiny_poly)]
pub use fq_tiny::Coeff;

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
