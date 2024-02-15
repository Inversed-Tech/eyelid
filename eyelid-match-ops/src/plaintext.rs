//! Iris matching operations on raw bit vectors.

use bitvec::{mem::bits_of, prelude::*};

use super::{IRIS_BIT_LENGTH, IRIS_MATCH_DENOMINATOR, IRIS_MATCH_NUMERATOR};

/// An iris code: the iris data from an iris scan.
/// A fixed-length bit array which is long enough to hold at least [`IRIS_BIT_LENGTH`] bits.
///
/// The array is rounded up to the next full `usize`, so it might contain some unused bits at the
/// end.
///
/// TODO: turn this into a wrapper struct, so the compiler checks IrisCode and IrisMask are used
///       correctly.
pub type IrisCode = BitArr![for IRIS_BIT_LENGTH];

/// An iris mask: the occlusion data from an iris scan.
/// See [`IrisCode`] for details.
///
/// TODO: turn this into a wrapper struct, so the compiler checks IrisCode and IrisMask are used
///       correctly.
pub type IrisMask = IrisCode;

/// Returns true if `eye_a` and `eye_b` have enough identical bits to meet the threshold,
/// after masking with `mask_a` and `mask_b`.
///
/// # Performance
///
/// This function takes references to avoid memory copies.
///
/// # TODO
///
/// - split this up into functions and test/benchmark them.
pub fn is_iris_match(
    eye_a: &IrisCode,
    mask_a: &IrisMask,
    eye_b: &IrisCode,
    mask_b: &IrisMask,
) -> bool {
    // Make sure iris codes and masks are the same size.
    // Performance: static assertions are checked at compile time.
    // TODO: I'm pretty sure the compiler already checks this as part of `&` or `^`,
    //       but I need to make sure.
    const_assert_eq!(bits_of::<IrisCode>(), bits_of::<IrisMask>());

    // Make sure there are no unused bits.
    // TODO: check unused bits are ignored in the tests instead.
    const_assert_eq!(bits_of::<IrisCode>(), IRIS_BIT_LENGTH);

    // Masking is applied to both iris codes before matching.
    //
    // TODO: benchmark these stack allocations:
    // - on the heap (using BitBox)
    // - on the heap using scratch memory that is allocated once, then passed to this function
    let unmasked = *mask_a & mask_b;
    let differences = (*eye_a ^ eye_b) & unmasked;

    // A successful match has enough matching unmasked bits to reach the match threshold.
    //
    // Convert to bit counts.
    let unmasked = unmasked.count_ones();
    let differences = differences.count_ones();

    // Make sure the threshold calculation can't overflow.
    // `IRIS_BIT_LENGTH` is the highest possible value of `matching` and `unmasked`.
    const_assert!(usize::MAX / IRIS_BIT_LENGTH > IRIS_MATCH_DENOMINATOR);
    const_assert!(usize::MAX / IRIS_BIT_LENGTH > IRIS_MATCH_NUMERATOR);

    // And compare with the threshold.
    differences * IRIS_MATCH_DENOMINATOR <= unmasked * IRIS_MATCH_NUMERATOR
}
