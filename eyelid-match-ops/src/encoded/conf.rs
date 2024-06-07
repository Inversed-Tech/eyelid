//! Encoding scheme configurations.

use ark_ff::{One, Zero};
use num_bigint::BigUint;

use crate::{
    encoded::MatchError, iris::conf::IrisConf, primitives::poly::PolyConf, FullRes, IrisBits,
    MiddleRes,
};

#[cfg(tiny_poly)]
use crate::TinyTest;

/// The dimensions of an encoding for an iris code, used for efficient matching.
pub trait EncodeConf {
    /// The configuration of iris code data.
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

// TODO: add a conf where EyeConf and PlainConf are different, and test it.

impl EncodeConf for IrisBits {
    type EyeConf = IrisBits;
    type PlainConf = IrisBits;

    const ROWS_PER_BLOCK: usize = IrisBits::COLUMN_LEN;
}

// TODO: work out how to automatically apply these assertions to every trait impl.
// (Or every config type.)
//
// We can't have more rows per block than actual rows.
const_assert!(IrisBits::ROWS_PER_BLOCK <= IrisBits::COLUMN_LEN);
// Only full blocks are supported at the moment.
const_assert_eq!(
    IrisBits::NUM_BLOCKS * IrisBits::ROWS_PER_BLOCK,
    IrisBits::COLUMN_LEN
);
// Each block must be able to be encoded into the configured polynomial.
const_assert!(
    IrisBits::NUM_COLS_AND_PADS * IrisBits::ROWS_PER_BLOCK
        <= <<IrisBits as EncodeConf>::PlainConf as PolyConf>::MAX_POLY_DEGREE
);

impl EncodeConf for FullRes {
    type EyeConf = FullRes;
    type PlainConf = FullRes;

    const ROWS_PER_BLOCK: usize = 16;
}
const_assert!(FullRes::ROWS_PER_BLOCK <= FullRes::COLUMN_LEN);
const_assert_eq!(
    FullRes::NUM_BLOCKS * FullRes::ROWS_PER_BLOCK,
    FullRes::COLUMN_LEN
);
const_assert!(
    FullRes::NUM_COLS_AND_PADS * FullRes::ROWS_PER_BLOCK
        <= <<FullRes as EncodeConf>::PlainConf as PolyConf>::MAX_POLY_DEGREE
);

impl EncodeConf for MiddleRes {
    type EyeConf = MiddleRes;
    type PlainConf = MiddleRes;

    const ROWS_PER_BLOCK: usize = 8;
}
const_assert!(MiddleRes::ROWS_PER_BLOCK <= MiddleRes::COLUMN_LEN);
const_assert_eq!(
    MiddleRes::NUM_BLOCKS * MiddleRes::ROWS_PER_BLOCK,
    MiddleRes::COLUMN_LEN
);
const_assert!(
    MiddleRes::NUM_COLS_AND_PADS * MiddleRes::ROWS_PER_BLOCK
        <= <<MiddleRes as EncodeConf>::PlainConf as PolyConf>::MAX_POLY_DEGREE
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
