//! Encoding scheme configurations.

use ark_ff::PrimeField;
use num_bigint::BigUint;

use crate::{encoded::MatchError, primitives::poly::PolyConf, FullRes, IrisBits, IrisConf};

#[cfg(tiny_poly)]
use crate::TinyTest;

/// An encoding for an iris code.
pub trait EncodeConf {
    /// The configuration of iris code data.
    type EyeConf: IrisConf;

    /// The configuration of plaintext polynomials.
    type PlainConf: PolyConf;

    /// The type of the coefficients of plaintext polynomials.
    //
    // TODO: use associated type defaults when they stabilise:
    // <https://github.com/rust-lang/rust/issues/29661>
    type PlainCoeff: PrimeField;

    // Divide iris codes into blocks that can each fit into a polynomial.
    /// The number of rows in each block: `s`
    const ROWS_PER_BLOCK: usize = 10;

    /// The number of blocks necessary to hold all rows of the code.
    const NUM_BLOCKS: usize = Self::EyeConf::COLUMN_LEN / Self::ROWS_PER_BLOCK;

    /// The number of columns plus padding for rotations: δ = k + v - u
    const NUM_COLS_AND_PADS: usize = Self::EyeConf::COLUMNS + 2 * Self::EyeConf::ROTATION_LIMIT;

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
}

// TODO: add a conf where EyeConf and PlainConf are different, and test it.

impl EncodeConf for IrisBits {
    type EyeConf = IrisBits;
    type PlainConf = IrisBits;
    type PlainCoeff = <Self::PlainConf as PolyConf>::Coeff;

    const ROWS_PER_BLOCK: usize = 20;
}

// TODO: work out how to automatically apply these assertions to every trait impl.
// (Or every config type.)
//
// Only full blocks are supported at the moment.
const_assert_eq!(
    IrisBits::NUM_BLOCKS * IrisBits::ROWS_PER_BLOCK,
    IrisBits::COLUMN_LEN
);

impl EncodeConf for FullRes {
    type EyeConf = FullRes;
    type PlainConf = FullRes;
    type PlainCoeff = <Self::PlainConf as PolyConf>::Coeff;

    const ROWS_PER_BLOCK: usize = 10;
}
const_assert_eq!(
    FullRes::NUM_BLOCKS * FullRes::ROWS_PER_BLOCK,
    FullRes::COLUMN_LEN
);

#[cfg(tiny_poly)]
impl EncodeConf for TinyTest {
    type EyeConf = TinyTest;
    type PlainConf = TinyTest;
    type PlainCoeff = <Self::PlainConf as PolyConf>::Coeff;

    const ROWS_PER_BLOCK: usize = 2;
}

/// This module avoids repeating `#[cfg(tiny_poly)]` for each assertion.
#[cfg(tiny_poly)]
mod tiny_test_asserts {
    use super::*;

    const_assert_eq!(
        TinyTest::NUM_BLOCKS * TinyTest::ROWS_PER_BLOCK,
        TinyTest::COLUMN_LEN
    );
}
