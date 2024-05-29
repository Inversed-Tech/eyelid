//! The underlying integer field.
//!
//! Outside this module, use [`PolyConf::Coeff`] instead of [`Fq79`] or `FqTiny`.
//! This automatically enables CI tests on both fields.

pub use fq79::Fq79;
pub use fq79bn::Fq79bn;

// Doc links only
#[allow(unused_imports)]
use crate::primitives::poly::PolyConf;

#[cfg(tiny_poly)]
pub use fq_tiny::Fq4 as FqTiny;

mod fq79;
mod fq79bn;

#[cfg(tiny_poly)]
mod fq_tiny;
