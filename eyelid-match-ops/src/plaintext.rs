//! Iris matching operations on raw bit vectors.

use crate::iris::conf::{IrisCode, IrisConf, IrisMask};

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

/// Returns the 1D index of a bit from 2D indices.
pub fn index_1d<const IRIS_COLUMN_LEN: usize>(row_i: usize, col_i: usize) -> usize {
    col_i * IRIS_COLUMN_LEN + row_i
}

/// Rotates the iris code by the given amount along the second dimension.
#[must_use = "rotations do nothing unless you assign them to a variable"]
#[allow(clippy::cast_sign_loss)]
pub fn rotate<C: IrisConf, const STORE_ELEM_LEN: usize>(
    mut code: IrisCode<STORE_ELEM_LEN>,
    amount: isize,
) -> IrisCode<STORE_ELEM_LEN> {
    if amount < 0 {
        code.rotate_left((-amount) as usize * C::COLUMN_LEN);
    } else {
        code.rotate_right(amount as usize * C::COLUMN_LEN);
    }
    code
}

/// Returns true if `eye_new` and `eye_store` have enough identical bits to meet the threshold,
/// after masking with `mask_new` and `mask_store`, and rotating from
/// [`-ROTATION_LIMIT..ROTATION_LIMIT`](IrisConf::ROTATION_LIMIT).
///
/// # Performance
///
/// This function takes references to avoid memory copies, which would otherwise be silent.
/// ([`IrisCode`] and [`IrisMask`] are [`Copy`] types.)
#[must_use = "matching does nothing unless you check its result"]
#[allow(clippy::cast_possible_wrap)]
pub fn is_iris_match<C: IrisConf, const STORE_ELEM_LEN: usize>(
    eye_new: &IrisCode<STORE_ELEM_LEN>,
    mask_new: &IrisMask<STORE_ELEM_LEN>,
    eye_store: &IrisCode<STORE_ELEM_LEN>,
    mask_store: &IrisMask<STORE_ELEM_LEN>,
) -> bool {
    // Start comparing columns at rotation -IRIS_ROTATION_LIMIT.
    // TODO:
    // - Avoid these copies and rotations by comparing bit indexes with an offset and modulus.
    // - If smaller rotations are more likely to exit early, start with them first.
    let mut eye_store = *eye_store;
    let mut mask_store = *mask_store;

    // These constant are tiny compared to isize, so they will never wrap.
    eye_store = rotate::<C, STORE_ELEM_LEN>(eye_store, -(C::ROTATION_LIMIT as isize));
    mask_store = rotate::<C, STORE_ELEM_LEN>(mask_store, -(C::ROTATION_LIMIT as isize));

    for _rotation in 0..C::ROTATION_COMPARISONS {
        /*dbg!(
            "rotation: ",
            -(C::ROTATION_LIMIT as isize) + _rotation as isize
        );*/

        // TODO:
        // - Make sure iris codes and masks are the same size.
        // - Check unused bits are ignored in the tests.

        // Masking is applied to both iris codes before matching.
        //
        // TODO: benchmark these stack allocations:
        // - on the heap (using BitBox)
        // - on the heap using scratch memory that is allocated once, then passed to this function
        let unmasked = *mask_new & mask_store;
        let raw_differences = *eye_new ^ eye_store;
        let differences = raw_differences & unmasked;

        // A successful match has enough matching unmasked bits to reach the match threshold.
        //
        // Convert to bit counts.
        let unmasked = unmasked.count_ones();
        let differences = differences.count_ones();

        // TODO:
        // - Make sure the threshold calculation can't overflow.
        // Currently this is only tested on the data used in debug builds.

        // And compare with the threshold.
        if differences * C::MATCH_DENOMINATOR <= unmasked * C::MATCH_NUMERATOR {
            return true;
        }

        // Move to the next highest column rotation.
        // TODO:
        // - Make this initial rotation part of the stored encoding.
        // - If smaller rotations are more likely to exit early, start with them first.
        eye_store = rotate::<C, STORE_ELEM_LEN>(eye_store, 1);
        mask_store = rotate::<C, STORE_ELEM_LEN>(mask_store, 1);
    }

    false
}
