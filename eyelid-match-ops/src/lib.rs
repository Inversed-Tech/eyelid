//! Iris code matching operations library.
//!
//! Masking is applied to both iris codes before matching:
//! ```text
//! unmasked = mask_a & mask_b
//! matching = (eye_a ^ eye_b) & unmasked
//! ```
//!
//! A successful match has:
//! ```text
//! |matching| / |unmasked| >= IRIS_MATCH_NUMERATOR / IRIS_MATCH_DENOMINATOR
//! ```
//!
//! This library has 3 core modules:
//! [`plaintext`]: operations on raw bit vectors,
//! [`encoded`]: the same operations on polynomial-encoded bit vectors,
//! [`encrypted`]: the same operations on fully homomorphic encrypted, polynomial-encoded bit
//!                vectors.

#[macro_use]
extern crate static_assertions;

pub mod encoded;
pub mod encrypted;
pub mod plaintext;

/// The length of a raw iris code or iris mask in bits.
/// Most users have two of these codes, for their left and right eyes.
pub const IRIS_BIT_LENGTH: usize = 2 * 20 * 160 * 2;

/// The numerator of the bit match threshold for a successful iris match.
pub const IRIS_MATCH_NUMERATOR: usize = 36;

/// The denominator of the bit match threshold for a successful iris match.
pub const IRIS_MATCH_DENOMINATOR: usize = 100;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
