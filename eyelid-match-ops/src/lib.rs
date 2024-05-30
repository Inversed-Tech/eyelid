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

// TODO: delete these constants after the crate has been updated to use IrisConf generic parameters.
use iris::conf::IrisConf;

/// The number of rows in a raw iris code or iris mask, in bits.
pub const IRIS_COLUMN_LENGTH: usize = IrisBits::COLUMN_LEN;

/// The number of columns in a raw iris code or iris mask, in bits.
pub const IRIS_COLUMNS: usize = IrisBits::COLUMNS;

/// The length of a raw iris code or iris mask, in bits.
/// Most users have two of these codes, for their left and right eyes.
pub const IRIS_BIT_LENGTH: usize = IrisBits::DATA_BIT_LEN;

/// The rotation limit when comparing irises.
/// Each column is compared to the [`IRIS_ROTATION_LIMIT`] columns to its left and right.
pub const IRIS_ROTATION_LIMIT: usize = IrisBits::ROTATION_LIMIT;

/// The number of rotations used when comparing irises.
/// This includes the comparison with no rotations.
pub const IRIS_ROTATION_COMPARISONS: usize = IrisBits::ROTATION_COMPARISONS;

/// The numerator of the bit match threshold for a successful iris match.
pub const IRIS_MATCH_NUMERATOR: usize = IrisBits::MATCH_NUMERATOR;

/// The denominator of the bit match threshold for a successful iris match.
pub const IRIS_MATCH_DENOMINATOR: usize = IrisBits::MATCH_DENOMINATOR;
