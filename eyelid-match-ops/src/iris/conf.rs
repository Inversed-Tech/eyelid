//! Base iris configurations, shared by all encoding and encryption schemes.
//!
//! These parameters are from the Inversed Tech report "Polynomial Encodings for FHE Relative Hamming Comparison v2".

use std::mem::size_of;

use bitvec::{mem::elts, prelude::BitArray};

use crate::{FullRes, IrisBits, MiddleRes};

#[cfg(tiny_poly)]
use crate::TinyTest;

/// The dimensions and matching rules for the entire iris code.
pub trait IrisConf {
    /// The number of rows in an iris code or iris mask.
    //
    // TODO: rename to `ROWS`
    const COLUMN_LEN: usize;

    /// The number of columns in an iris code or mask, `k`.
    const COLUMNS: usize;

    /// The length of an iris code or mask.
    const DATA_BIT_LEN: usize = Self::COLUMN_LEN * Self::COLUMNS;

    /// The length of the underlying storage for an iris code or mask.
    const STORE_ELEM_LEN: usize = elts::<IrisStore>(Self::DATA_BIT_LEN);

    /// The rotation limits when comparing irises, `v` and `u = -v`.
    /// Each column is compared to the [`ROTATION_LIMIT`](Self::ROTATION_LIMIT) columns to its left and right.
    const ROTATION_LIMIT: usize;

    /// The number of rotations used when comparing irises, `v - u + 1`.
    /// This includes the comparison with no rotation.
    const ROTATION_COMPARISONS: usize = Self::ROTATION_LIMIT * 2 + 1;

    /// The numerator of the bit match threshold for a successful iris match.
    /// The default match threshold is 36%.
    const MATCH_NUMERATOR: usize = 36;

    /// The denominator of the bit match threshold for a successful iris match.
    /// The default match threshold is 36%.
    const MATCH_DENOMINATOR: usize = 100;
}

/// A type alias for the underlying array element type.
/// Not currently configurable via the trait.
type IrisStore = usize;

/// An iris code: the iris data from an iris scan.
/// A fixed-length bit array which is long enough to hold at least [`IrisConf::DATA_BIT_LEN`] bits.
///
/// The encoding of an iris code is arbitrary, because we just check for matching bits.
///
/// The array is rounded up to the next full `usize`, so it might contain some unused bits at the
/// end.
///
/// TODO: turn this into a wrapper struct, so the compiler checks IrisCode and IrisMask are used
///       correctly.
pub type IrisCode<const STORE_ELEM_LEN: usize> = BitArray<[IrisStore; STORE_ELEM_LEN]>;

/// An iris mask: the occlusion data from an iris scan.
/// See [`IrisCode`] for details.
///
/// The encoding of an iris mask is `1` for a comparable bit, and `0` for a masked bit.
///
/// TODO: turn this into a wrapper struct, so the compiler checks IrisCode and IrisMask are used
///       correctly.
pub type IrisMask<const STORE_ELEM_LEN: usize> = BitArray<[IrisStore; STORE_ELEM_LEN]>;

impl IrisConf for IrisBits {
    const COLUMN_LEN: usize = 80;
    const COLUMNS: usize = 160;
    const ROTATION_LIMIT: usize = 15;
}
// TODO: work out how to automatically apply these assertions to every trait impl.
// (Or every config type.)
//
// There must be enough bits to store the underlying data.
const_assert!(IrisBits::DATA_BIT_LEN >= IrisBits::COLUMN_LEN * IrisBits::COLUMNS);
const_assert!(IrisBits::STORE_ELEM_LEN * size_of::<IrisStore>() * 8 >= IrisBits::DATA_BIT_LEN);
// Rotating more than the number of columns is redundant.
const_assert!(IrisBits::ROTATION_COMPARISONS <= IrisBits::COLUMNS);
// The match fraction should be between 0 and 1.
const_assert!(IrisBits::MATCH_NUMERATOR <= IrisBits::MATCH_DENOMINATOR);
const_assert!(IrisBits::MATCH_DENOMINATOR > 0);

impl IrisConf for FullRes {
    const COLUMN_LEN: usize = 10;
    const COLUMNS: usize = 160;
    const ROTATION_LIMIT: usize = IrisBits::ROTATION_LIMIT;
}
const_assert!(FullRes::DATA_BIT_LEN >= FullRes::COLUMN_LEN * FullRes::COLUMNS);
const_assert!(FullRes::STORE_ELEM_LEN * size_of::<IrisStore>() * 8 >= FullRes::DATA_BIT_LEN);
const_assert!(FullRes::ROTATION_COMPARISONS <= FullRes::COLUMNS);
const_assert!(FullRes::MATCH_NUMERATOR <= FullRes::MATCH_DENOMINATOR);
const_assert!(FullRes::MATCH_DENOMINATOR > 0);

impl IrisConf for MiddleRes {
    const COLUMN_LEN: usize = 5;
    const COLUMNS: usize = 80;
    const ROTATION_LIMIT: usize = IrisBits::ROTATION_LIMIT;
}
const_assert!(MiddleRes::DATA_BIT_LEN >= MiddleRes::COLUMN_LEN * MiddleRes::COLUMNS);
const_assert!(MiddleRes::STORE_ELEM_LEN * size_of::<IrisStore>() * 8 >= MiddleRes::DATA_BIT_LEN);
const_assert!(MiddleRes::ROTATION_COMPARISONS <= MiddleRes::COLUMNS);
const_assert!(MiddleRes::MATCH_NUMERATOR <= MiddleRes::MATCH_DENOMINATOR);
const_assert!(MiddleRes::MATCH_DENOMINATOR > 0);

#[cfg(tiny_poly)]
impl IrisConf for TinyTest {
    const COLUMN_LEN: usize = 2;
    const COLUMNS: usize = 3;
    const ROTATION_LIMIT: usize = 1;
}

/// This module avoids repeating `#[cfg(tiny_poly)]` for each assertion.
#[cfg(tiny_poly)]
mod tiny_test_asserts {
    use super::*;

    const_assert!(TinyTest::DATA_BIT_LEN >= TinyTest::COLUMN_LEN * TinyTest::COLUMNS);
    const_assert!(TinyTest::STORE_ELEM_LEN * size_of::<IrisStore>() * 8 >= TinyTest::DATA_BIT_LEN);
    const_assert!(TinyTest::ROTATION_COMPARISONS <= TinyTest::COLUMNS);
    const_assert!(TinyTest::MATCH_NUMERATOR <= TinyTest::MATCH_DENOMINATOR);
    const_assert!(TinyTest::MATCH_DENOMINATOR > 0);
}
