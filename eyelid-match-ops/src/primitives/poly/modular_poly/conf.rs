//! Fixed parameters for modular polynomial types.

use std::fmt::Debug;

use ark_ff::{PrimeField, Zero};
use lazy_static::lazy_static;

use crate::{
    encoded::{conf::LargeRes, EncodeConf, FullRes, MiddleRes},
    primitives::poly::{
        fq::{Fq123, Fq123bn},
        Fq66, Fq66bn, Fq79, Fq79bn,
    },
    FullBits, MiddleBits,
};

#[cfg(tiny_poly)]
use crate::{
    primitives::poly::fq::{FqTiny, FqTinybn},
    TinyTest,
};

/// Fixed polynomial parameters.
///
/// Polynomials with different parameters are incompatible.
pub trait PolyConf: Copy + Clone + Debug + Eq + PartialEq {
    /// The maximum exponent in the polynomial.
    const MAX_POLY_DEGREE: usize;

    /// The type of the polynomial coefficient.
    type Coeff: PrimeField;

    /// The zero coefficient as a static constant value.
    ///
    /// # Usage
    ///
    /// Return `&PolyConf::COEFF_ZERO` from a function that returns a reference to `Coeff::zero()`.
    ///
    /// Only use this constant when you need a long-lived reference to a zero coefficient value.
    /// The compiler will tell you, with errors like:
    /// > cannot return reference to a temporary value
    /// > returns a reference to data owned by the current function
    ///
    /// Typically, `Coeff::zero()` is more readable and efficient.
    fn coeff_zero() -> &'static Self::Coeff;
}

impl PolyConf for LargeRes {
    const MAX_POLY_DEGREE: usize = FullBits::BLOCK_AND_PADS_BIT_LEN.next_power_of_two();

    type Coeff = Fq123;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ123_ZERO
    }
}
// The polynomial must have enough coefficients to store the underlying iris data.
//const_assert!(FullRes::MAX_POLY_DEGREE >= FullBits::BLOCK_AND_PADS_BIT_LEN);
// The degree must be a power of two.
const_assert!(LargeRes::MAX_POLY_DEGREE.count_ones() == 1);

impl PolyConf for LargeResBN {
    // This degree requires a larger modulus, Fq79 doesn't work
    const MAX_POLY_DEGREE: usize = LargeRes::MAX_POLY_DEGREE;

    type Coeff = Fq123bn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ123_BN_ZERO
    }
}
// The polynomial must have enough coefficients to store the underlying iris data.
const_assert!(LargeResBN::MAX_POLY_DEGREE >= FullBits::BLOCK_AND_PADS_BIT_LEN);
// The degree must be a power of two.
const_assert!(LargeResBN::MAX_POLY_DEGREE.count_ones() == 1);

impl PolyConf for FullRes {
    const MAX_POLY_DEGREE: usize = FullBits::BLOCK_AND_PADS_BIT_LEN.next_power_of_two();

    type Coeff = Fq79;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ79_ZERO
    }
}
// The polynomial must have enough coefficients to store the underlying iris data.
const_assert!(FullRes::MAX_POLY_DEGREE >= FullBits::BLOCK_AND_PADS_BIT_LEN);
// The degree must be a power of two.
const_assert!(FullRes::MAX_POLY_DEGREE.count_ones() == 1);

impl PolyConf for FullResBN {
    // This degree requires a larger modulus, Fq79 doesn't work
    const MAX_POLY_DEGREE: usize = FullRes::MAX_POLY_DEGREE;

    type Coeff = Fq79bn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ79_BN_ZERO
    }
}
// The polynomial must have enough coefficients to store the underlying iris data.
const_assert!(FullResBN::MAX_POLY_DEGREE >= FullBits::BLOCK_AND_PADS_BIT_LEN);
// The degree must be a power of two.
const_assert!(FullResBN::MAX_POLY_DEGREE.count_ones() == 1);

impl PolyConf for MiddleRes {
    const MAX_POLY_DEGREE: usize = MiddleBits::BLOCK_AND_PADS_BIT_LEN.next_power_of_two();

    type Coeff = Fq66;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ66_ZERO
    }
}
const_assert!(MiddleRes::MAX_POLY_DEGREE >= MiddleBits::BLOCK_AND_PADS_BIT_LEN);
const_assert!(MiddleRes::MAX_POLY_DEGREE.count_ones() == 1);

impl PolyConf for MiddleResBN {
    const MAX_POLY_DEGREE: usize = MiddleRes::MAX_POLY_DEGREE;

    type Coeff = Fq66bn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ66_BN_ZERO
    }
}
const_assert!(MiddleResBN::MAX_POLY_DEGREE >= MiddleBits::BLOCK_AND_PADS_BIT_LEN);
const_assert!(MiddleResBN::MAX_POLY_DEGREE.count_ones() == 1);

#[cfg(tiny_poly)]
impl PolyConf for TinyTest {
    const MAX_POLY_DEGREE: usize = TinyTest::BLOCK_AND_PADS_BIT_LEN.next_power_of_two();

    type Coeff = FqTiny;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ_TINY_ZERO
    }
}

#[cfg(tiny_poly)]
impl PolyConf for TinyTestBN {
    const MAX_POLY_DEGREE: usize = TinyTest::MAX_POLY_DEGREE;

    // TODO: find a coefficient that works here
    type Coeff = FqTinybn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ_TINY_BN_ZERO
    }
}

/// This module avoids repeating `#[cfg(tiny_poly)]` for each assertion.
#[cfg(tiny_poly)]
mod tiny_test_asserts {
    use super::*;
    const_assert!(TinyTest::MAX_POLY_DEGREE >= TinyTest::BLOCK_AND_PADS_BIT_LEN);
    const_assert!(TinyTest::MAX_POLY_DEGREE.count_ones() == 1);
    const_assert!(TinyTestBN::MAX_POLY_DEGREE >= TinyTest::BLOCK_AND_PADS_BIT_LEN);
    const_assert!(TinyTestBN::MAX_POLY_DEGREE.count_ones() == 1);
}

#[cfg(tiny_poly)]
lazy_static! {
    /// The zero coefficient as a static constant value.
    static ref FQ_TINY_ZERO: FqTiny = FqTiny::zero();
    static ref FQ_TINY_BN_ZERO: FqTinybn = FqTinybn::zero();
}

/// Large resolution polynomial parameters for lifted coefficients.
///
/// These are the parameters for large resolution, since FullRes was not enough.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LargeResBN;

/// Full resolution polynomial parameters for lifted coefficients.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FullResBN;

/// Middle resolution polynomial parameters for lifted coefficients.
///
/// These are the parameters for middle resolution, according to the Inversed Tech report.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MiddleResBN;

/// Tiny test polynomials for lifted coefficients, used for finding edge cases in tests.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.
#[cfg(tiny_poly)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TinyTestBN;

// TODO: try generic_singleton and see if it performs better:
// <https://docs.rs/generic_singleton/0.5.0/generic_singleton/macro.get_or_init_thread_local.html>
lazy_static! {
    /// The zero coefficient as a static constant value.
    static ref FQ123_ZERO: Fq123 = Fq123::zero();

    /// The zero coefficient as a static constant value.
    static ref FQ123_BN_ZERO: Fq123bn = Fq123bn::zero();

    /// The zero coefficient as a static constant value.
    static ref FQ79_ZERO: Fq79 = Fq79::zero();

    /// The zero coefficient as a static constant value.
    static ref FQ79_BN_ZERO: Fq79bn = Fq79bn::zero();

    /// The zero coefficient as a static constant value.
    static ref FQ66_ZERO: Fq66 = Fq66::zero();

    /// The zero coefficient as a static constant value.
    static ref FQ66_BN_ZERO: Fq66bn = Fq66bn::zero();
}
