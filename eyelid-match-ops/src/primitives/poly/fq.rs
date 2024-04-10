//! The underlying integer field.
//!
//! Outside this module, use [`fq::Coeff`](Coeff) and [`fq::MAX_POLY_DEGREE`](MAX_POLY_DEGREE) instead of `fq79` or `fq_tiny`.
//! This automatically enables CI tests on both fields.

mod fq79;
mod fq_tiny;

#[cfg(not(tiny_poly))]
pub use fq79::{Coeff, MAX_POLY_DEGREE};

// Temporarily switch to this tiny field to make test errors easier to debug:
// ```no_run
// RUSTFLAGS="--cfg tiny_poly" cargo test
// RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
// ```
#[cfg(tiny_poly)]
pub use fq_tiny::{Coeff, MAX_POLY_DEGREE};