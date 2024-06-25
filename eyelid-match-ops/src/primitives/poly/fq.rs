//! The underlying integer field.
//!
//! Outside this module, use [`PolyConf::Coeff`] instead of [`Fq79`] or `FqTiny`.
//! This automatically enables CI tests on both fields.

pub use fq123::Fq123;
pub use fq123bn::Fq123bn;

pub use fq79::Fq79;
pub use fq79bn::Fq79bn;

pub use fq66::Fq66;
pub use fq66bn::Fq66bn;

// Doc links only
#[allow(unused_imports)]
use crate::primitives::poly::PolyConf;

#[cfg(tiny_poly)]
pub use fq_tiny::Fq4 as FqTiny;

#[cfg(tiny_poly)]
pub use fq_tiny_bn::Fq4 as FqTinybn;

mod fq123;
mod fq123bn;

mod fq79;
mod fq79bn;

mod fq66;
mod fq66bn;

#[cfg(tiny_poly)]
mod fq_tiny;

#[cfg(tiny_poly)]
mod fq_tiny_bn;
