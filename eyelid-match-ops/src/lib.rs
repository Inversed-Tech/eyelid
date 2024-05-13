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

/// The number of rows in a raw iris code or iris mask, in bits.
pub const IRIS_COLUMN_LENGTH: usize = 80;

/// The number of columns in a raw iris code or iris mask, in bits.
pub const IRIS_COLUMNS: usize = 160;

/// The length of a raw iris code or iris mask, in bits.
/// Most users have two of these codes, for their left and right eyes.
pub const IRIS_BIT_LENGTH: usize = IRIS_COLUMN_LENGTH * IRIS_COLUMNS;

/// The rotation limit when comparing irises.
/// Each column is compared to the [`IRIS_ROTATION_LIMIT`] columns to its left and right.
pub const IRIS_ROTATION_LIMIT: usize = 15;

/// The number of rotations used when comparing irises.
/// This includes the comparison with no rotations.
pub const IRIS_ROTATION_COMPARISONS: usize = IRIS_ROTATION_LIMIT * 2 + 1;

/// The numerator of the bit match threshold for a successful iris match.
pub const IRIS_MATCH_NUMERATOR: usize = 36;

/// The denominator of the bit match threshold for a successful iris match.
pub const IRIS_MATCH_DENOMINATOR: usize = 100;
