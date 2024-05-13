//! Iris configurations for encoding and encryption schemes.

use crate::{FullRes, IrisBits};

#[cfg(tiny_poly)]
use crate::TinyTest;

/// The dimensions and matching rules for an iris code.
pub trait IrisConf {
    /// The number of rows in an iris code or iris mask.
    const COLUMN_LENGTH: usize;

    /// The number of columns in an iris code or iris mask.
    const COLUMNS: usize;

    /// The length of an iris code or iris mask.
    const BIT_LENGTH: usize = Self::COLUMN_LENGTH * Self::COLUMNS;

    /// The rotation limit when comparing irises.
    /// Each column is compared to the [`ROTATION_LIMIT`](Self::ROTATION_LIMIT) columns to its left and right.
    const ROTATION_LIMIT: usize;

    /// The number of rotations used when comparing irises.
    /// This includes the comparison with no rotation.
    const ROTATION_COMPARISONS: usize = Self::ROTATION_LIMIT * 2 + 1;

    /// The numerator of the bit match threshold for a successful iris match.
    /// The default match threshold is 36%.
    const MATCH_NUMERATOR: usize = 36;

    /// The denominator of the bit match threshold for a successful iris match.
    /// The default match threshold is 36%.
    const MATCH_DENOMINATOR: usize = 100;
}

impl IrisConf for IrisBits {
    const COLUMN_LENGTH: usize = 80;
    const COLUMNS: usize = 160;
    const ROTATION_LIMIT: usize = 15;
}
// There must be enough bits to store the underlying data.
const_assert!(IrisBits::BIT_LENGTH >= IrisBits::COLUMN_LENGTH * IrisBits::COLUMNS);
// Rotating more than the number of columns is redundant.
const_assert!(IrisBits::ROTATION_COMPARISONS <= IrisBits::COLUMNS);
// The match fraction should be strictly between 0 and 1.
const_assert!(IrisBits::MATCH_NUMERATOR <= IrisBits::MATCH_DENOMINATOR);
const_assert!(IrisBits::MATCH_DENOMINATOR > 0);

impl IrisConf for FullRes {
    const COLUMN_LENGTH: usize = 10;
    const COLUMNS: usize = 160;
    const ROTATION_LIMIT: usize = IrisBits::ROTATION_LIMIT;
}
const_assert!(FullRes::BIT_LENGTH >= FullRes::COLUMN_LENGTH * FullRes::COLUMNS);
const_assert!(FullRes::ROTATION_COMPARISONS <= FullRes::COLUMNS);
const_assert!(FullRes::MATCH_NUMERATOR <= FullRes::MATCH_DENOMINATOR);
const_assert!(FullRes::MATCH_DENOMINATOR > 0);

#[cfg(tiny_poly)]
impl IrisConf for TinyTest {
    const COLUMN_LENGTH: usize = 2;
    const COLUMNS: usize = 3;
    const ROTATION_LIMIT: usize = 1;
}
#[cfg(tiny_poly)]
mod tiny_test_asserts {
    use super::*;

    const_assert!(TinyTest::BIT_LENGTH >= TinyTest::COLUMN_LENGTH * TinyTest::COLUMNS);
    const_assert!(TinyTest::ROTATION_COMPARISONS <= TinyTest::COLUMNS);
    const_assert!(TinyTest::MATCH_NUMERATOR <= TinyTest::MATCH_DENOMINATOR);
    const_assert!(TinyTest::MATCH_DENOMINATOR > 0);
}
