//! Fixed parameters for modular polynomial types.

use std::fmt::Debug;

use ark_ff::{PrimeField, Zero};
use lazy_static::lazy_static;

use crate::primitives::poly::Fq79;
use crate::primitives::poly::fq::Fq79bn;

#[cfg(tiny_poly)]
use crate::primitives::poly::fq::FqTiny;

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

/// Iris bit length polynomial parameters.
///
/// This uses the full number of iris bits, which gives an upper bound on benchmarks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IrisBits;

impl PolyConf for IrisBits {
    const MAX_POLY_DEGREE: usize = crate::IRIS_BIT_LENGTH.next_power_of_two();

    type Coeff = Fq79;
    type CoeffBN = Fq79bn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ79_ZERO
    }
}

lazy_static! {
    /// The zero coefficient as a static constant value.
    static ref FQ79_ZERO: Fq79 = Fq79::zero();
}

// TODO: try generic_singleton and see if it performs better:
// <https://docs.rs/generic_singleton/0.5.0/generic_singleton/macro.get_or_init_thread_local.html>

/// Full resolution polynomial parameters.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FullRes;

impl PolyConf for FullRes {
    const MAX_POLY_DEGREE: usize = 1024;

    type Coeff = Fq79;
    type CoeffBN = Fq79bn;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ79_ZERO
    }
}

/// Tiny test polynomials, used for finding edge cases in tests.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.
#[cfg(tiny_poly)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TinyTest;

#[cfg(tiny_poly)]
impl PolyConf for TinyTest {
    const MAX_POLY_DEGREE: usize = 8;

    type Coeff = FqTiny;
    type CoeffBN = FqTiny;

    fn coeff_zero() -> &'static Self::Coeff {
        &FQ_TINY_ZERO
    }
}

#[cfg(tiny_poly)]
lazy_static! {
    /// The zero coefficient as a static constant value.
    static ref FQ_TINY_ZERO: FqTiny = FqTiny::zero();
}
