//! Fixed parameters for modular polynomial types.

use std::fmt::Debug;

use ark_ff::{PrimeField, Zero};
use lazy_static::lazy_static;

use crate::{
    iris::conf::IrisConf,
    primitives::poly::{Fq66, Fq66bn, Fq79, Fq79bn},
    FullRes, IrisBits, MiddleRes,
};

#[cfg(tiny_poly)]
use crate::{primitives::poly::fq::FqTiny, TinyTest};

/// Fixed polynomial parameters.
///
/// Polynomials with different parameters are incompatible.
pub trait PolyConf: Copy + Clone + Debug + Eq + PartialEq {
    /// The maximum exponent in the polynomial.
    const MAX_POLY_DEGREE: usize;

    /// The type of the polynomial coefficient.
    type Coeff: PrimeField;
    /// The type of the lifted polynomial coefficient.
    type CoeffBN: PrimeField;

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

impl PolyConf for IrisBits {
    // This degree requires a larger modulus, Fq79 doesn't work
    const MAX_POLY_DEGREE: usize = IrisBits::DATA_BIT_LEN.next_power_of_two();

    type Coeff = Fq79;
    type CoeffBN = Fq79bn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ79_ZERO
    }
}
// The polynomial must have enough coefficients to store the underlying iris data.
const_assert!(IrisBits::MAX_POLY_DEGREE >= IrisBits::DATA_BIT_LEN);
// The degree must be a power of two.
const_assert!(IrisBits::MAX_POLY_DEGREE.count_ones() == 1);

// TODO: try generic_singleton and see if it performs better:
// <https://docs.rs/generic_singleton/0.5.0/generic_singleton/macro.get_or_init_thread_local.html>
lazy_static! {
    /// The zero coefficient as a static constant value.
    static ref FQ79_ZERO: Fq79 = Fq79::zero();
}

lazy_static! {
    /// The zero coefficient as a static constant value.
    static ref FQ66_ZERO: Fq66 = Fq66::zero();
}

// TODO: try generic_singleton and see if it performs better:
// <https://docs.rs/generic_singleton/0.5.0/generic_singleton/macro.get_or_init_thread_local.html>

impl PolyConf for FullRes {
    const MAX_POLY_DEGREE: usize = 2048;

    type Coeff = Fq79;
    type CoeffBN = Fq79bn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ79_ZERO
    }
}

impl PolyConf for MiddleRes {
    const MAX_POLY_DEGREE: usize = 1024;

    type Coeff = Fq66;
    type CoeffBN = Fq66bn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ66_ZERO
    }
}

/// Tiny test polynomials, used for finding edge cases in tests.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.
#[cfg(tiny_poly)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TinyTest;
const_assert!(FullRes::MAX_POLY_DEGREE >= FullRes::DATA_BIT_LEN);
const_assert!(FullRes::MAX_POLY_DEGREE.count_ones() == 1);

#[cfg(tiny_poly)]
impl PolyConf for TinyTest {
    const MAX_POLY_DEGREE: usize = 16;

    type Coeff = FqTiny;
    type CoeffBN = FqTiny;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ_TINY_ZERO
    }
}

/// This module avoids repeating `#[cfg(tiny_poly)]` for each assertion.
#[cfg(tiny_poly)]
mod tiny_test_asserts {
    use super::*;
    const_assert!(TinyTest::MAX_POLY_DEGREE >= TinyTest::DATA_BIT_LEN);
    const_assert!(TinyTest::MAX_POLY_DEGREE.count_ones() == 1);
}

#[cfg(tiny_poly)]
lazy_static! {
    /// The zero coefficient as a static constant value.
    static ref FQ_TINY_ZERO: FqTiny = FqTiny::zero();
}
