//! Iris matching operations on raw bit vectors.

use bitvec::{mem::bits_of, prelude::*};

use super::{
    IRIS_BIT_LENGTH, IRIS_COLUMN_LENGTH, IRIS_MATCH_DENOMINATOR, IRIS_MATCH_NUMERATOR,
    IRIS_ROTATION_COMPARISONS, IRIS_ROTATION_LIMIT,
};

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
/// after masking with `mask_a` and `mask_b`, and rotating from
/// [`-IRIS_ROTATION_LIMIT..IRIS_ROTATION_LIMIT`](IRIS_ROTATION_LIMIT).
///
/// # Performance
///
/// This function takes references to avoid memory copies.
/// The stored eye is an owned value, so it can be rotated without copying.
///
/// # TODO
///
/// - split this up into functions and test/benchmark them.
pub fn is_iris_match(
    eye_new: &IrisCode,
    mask_new: &IrisMask,
    mut eye_store: IrisCode,
    mut mask_store: IrisMask,
) -> bool {
    // Start comparing columns at rotation -IRIS_ROTATION_LIMIT.
    // TODO:
    // - Avoid the rotations by comparing bit indexes with an offset and modulus.
    // - If smaller rotations are more likely to exit early, start with them first.
    eye_store.rotate_left(IRIS_ROTATION_LIMIT * IRIS_COLUMN_LENGTH);
    mask_store.rotate_left(IRIS_ROTATION_LIMIT * IRIS_COLUMN_LENGTH);

    for _rotation in 0..IRIS_ROTATION_COMPARISONS {
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
        let unmasked = *mask_new & mask_store;
        let differences = (*eye_new ^ eye_store) & unmasked;

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
        if differences * IRIS_MATCH_DENOMINATOR <= unmasked * IRIS_MATCH_NUMERATOR {
            return true;
        }

        // Move to the next highest column rotation.
        // TODO:
        // - Make this initial rotation part of the stored encoding.
        // - If smaller rotations are more likely to exit early, start with them first.
        eye_store.rotate_right(IRIS_COLUMN_LENGTH);
        mask_store.rotate_right(IRIS_COLUMN_LENGTH);
    }

    false
}
