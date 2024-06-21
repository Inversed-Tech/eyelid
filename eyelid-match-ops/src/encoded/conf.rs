//! Encoding scheme configurations.

use ark_ff::{One, Zero};
use num_bigint::BigUint;

use crate::{
    encoded::MatchError, iris::conf::IrisConf, primitives::poly::PolyConf, FullBits, MiddleBits,
};

#[cfg(tiny_poly)]
use crate::TinyTest;

/// The dimensions of an encoding for an iris code, used for efficient matching.
pub trait EncodeConf {
    /// The configuration of iris code data.
    ///
    /// TODO: rename to EyeData and DataConf?
    type EyeConf: IrisConf;

    /// The configuration of plaintext polynomials.
    type PlainConf: PolyConf;

    /// Divide iris codes into blocks that can each fit into a polynomial.
    /// The number of rows in each block: `s`
    const ROWS_PER_BLOCK: usize;

    /// The number of iris bits in each block.
    const BLOCK_BIT_LEN: usize = Self::EyeConf::COLUMN_LEN * Self::ROWS_PER_BLOCK;

    /// The number of blocks necessary to hold all rows of the code.
    const NUM_BLOCKS: usize = Self::EyeConf::COLUMN_LEN / Self::ROWS_PER_BLOCK;

    /// The number of columns plus padding for rotations: δ = k + v - u
    const NUM_COLS_AND_PADS: usize = Self::EyeConf::COLUMNS + 2 * Self::EyeConf::ROTATION_LIMIT;

    /// The number of iris bits in each block.
    const BLOCK_AND_PADS_BIT_LEN: usize = Self::NUM_COLS_AND_PADS * Self::ROWS_PER_BLOCK;

    /// Convert a prime field element to a signed integer, assuming the range from all equal to all different bits.
    /// Out of range values return `Err(err)`.
    fn coeff_to_int(
        c: <Self::PlainConf as PolyConf>::Coeff,
        err: MatchError,
    ) -> Result<i64, MatchError>
    where
        BigUint: From<<Self::PlainConf as PolyConf>::Coeff>,
    {
        let res = if c
            <= <Self::PlainConf as PolyConf>::Coeff::from(Self::EyeConf::DATA_BIT_LEN as u64)
        {
            i64::try_from(BigUint::from(c)).map_err(|_| err)?
        } else {
            -i64::try_from(BigUint::from(-c)).map_err(|_| err)?
        };

        Ok(res)
    }

    /// Returns the `zero` value of a prime field element.
    fn coeff_zero() -> <Self::PlainConf as PolyConf>::Coeff {
        <Self::PlainConf as PolyConf>::Coeff::zero()
    }

    /// Returns the `one` value of a prime field element.
    fn coeff_one() -> <Self::PlainConf as PolyConf>::Coeff {
        <Self::PlainConf as PolyConf>::Coeff::one()
    }
}

impl EncodeConf for FullBits {
    type EyeConf = FullBits;
    type PlainConf = FullRes;

    const ROWS_PER_BLOCK: usize = 8;
}
// As in the report
const_assert_eq!(
    <<FullBits as EncodeConf>::PlainConf as PolyConf>::MAX_POLY_DEGREE,
    2048
);

// TODO: work out how to automatically apply these assertions to every trait impl.
// (Or every config type.)
//
// We can't have more rows per block than actual rows.
const_assert!(FullBits::ROWS_PER_BLOCK <= FullBits::COLUMN_LEN);
// Only full blocks are supported at the moment.
const_assert_eq!(
    FullBits::NUM_BLOCKS * FullBits::ROWS_PER_BLOCK,
    FullBits::COLUMN_LEN
);
// Each block must be able to be encoded into the configured polynomial.
const_assert!(
    FullBits::NUM_COLS_AND_PADS * FullBits::ROWS_PER_BLOCK
        <= <<FullBits as EncodeConf>::PlainConf as PolyConf>::MAX_POLY_DEGREE
);

impl EncodeConf for MiddleBits {
    type EyeConf = MiddleBits;
    type PlainConf = MiddleRes;

    const ROWS_PER_BLOCK: usize = 4;
}
// As in the report
const_assert_eq!(
    <<MiddleBits as EncodeConf>::PlainConf as PolyConf>::MAX_POLY_DEGREE,
    1024
);

const_assert!(MiddleBits::ROWS_PER_BLOCK <= MiddleBits::COLUMN_LEN);
const_assert_eq!(
    MiddleBits::NUM_BLOCKS * MiddleBits::ROWS_PER_BLOCK,
    MiddleBits::COLUMN_LEN
);
const_assert!(
    MiddleBits::NUM_COLS_AND_PADS * MiddleBits::ROWS_PER_BLOCK
        <= <<MiddleBits as EncodeConf>::PlainConf as PolyConf>::MAX_POLY_DEGREE
);

#[cfg(tiny_poly)]
impl EncodeConf for TinyTest {
    type EyeConf = TinyTest;
    type PlainConf = TinyTest;

    const ROWS_PER_BLOCK: usize = 1;
}

/// This module avoids repeating `#[cfg(tiny_poly)]` for each assertion.
#[cfg(tiny_poly)]
mod tiny_test_asserts {
    use super::*;

    const_assert!(TinyTest::ROWS_PER_BLOCK <= TinyTest::COLUMN_LEN);
    const_assert_eq!(
        TinyTest::NUM_BLOCKS * TinyTest::ROWS_PER_BLOCK,
        TinyTest::COLUMN_LEN
    );
    const_assert!(
        TinyTest::NUM_COLS_AND_PADS * TinyTest::ROWS_PER_BLOCK
            <= <<TinyTest as EncodeConf>::PlainConf as PolyConf>::MAX_POLY_DEGREE
    );
}

/// Large resolution polynomial parameters.
///
/// These are the parameters for large resolution, since FullRes was not enough.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LargeRes;

/// Full resolution polynomial parameters.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FullRes;

/// Middle resolution polynomial parameters.
///
/// These are the parameters for middle resolution, according to the Inversed Tech report.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MiddleRes;

/// The polynomial config used in tests.
//
// We use the full resolution by default, but TinyTest when cfg(tiny_poly) is set.
#[cfg(not(tiny_poly))]
pub type TestRes = FullRes;

/// The polynomial config used in tests.
///
/// Temporarily switch to this tiny field to make test errors easier to debug:
/// ```no_run
/// RUSTFLAGS="--cfg tiny_poly" cargo test
/// RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
/// ```
#[cfg(tiny_poly)]
pub type TestRes = TinyTest;
