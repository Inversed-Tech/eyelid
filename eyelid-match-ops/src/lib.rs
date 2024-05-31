//! Iris code matching operations library.
//!
//! Masking is applied to both iris codes before matching, then one of them is rotated. A
//! successful match has enough matching unmasked bits to reach the match threshold, in at least
//! one rotation.
//!
//! This library has 3 core modules:
//! [`plaintext`]: operations on raw bit vectors,
//! [`encoded`]: the same operations on polynomial-encoded bit vectors,
//! [`encrypted`]: the same operations on fully homomorphic encrypted, polynomial-encoded bit
//!                vectors.
//!
//! Configurations are in [`conf`] and [`iris`], and building blocks are in [`primitives`].

#[macro_use]
extern crate static_assertions;

pub mod conf;
pub mod encoded;
pub mod encrypted;
pub mod iris;
pub mod plaintext;
pub mod primitives;

pub use conf::{FullRes, IrisBits};

#[cfg(any(test, feature = "benchmark"))]
pub use conf::TestRes;

#[cfg(tiny_poly)]
pub use conf::TinyTest;
