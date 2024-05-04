//! Fixed parameters for modular polynomial types.

use std::fmt::Debug;

/// The polynomial config used in tests.
//
// We use the full resolution by default, but TinyTest when cfg(tiny_poly) is set.
#[cfg(not(tiny_poly))]
pub type TestRes = FullRes;

/// The polynomial config used in tests.
#[cfg(tiny_poly)]
pub type TestRes = TinyTest;

/// Fixed polynomial parameters.
///
/// Polynomials with different parameters are incompatible.
pub trait PolyConf: Copy + Clone + Debug + Eq + PartialEq {
    /// The maximum exponent in the polynomial.
    const MAX_POLY_DEGREE: usize;

    // TODO: add Coeff type
}

/// Iris bit length polynomial parameters.
///
/// This uses the full number of iris bits, which gives an upper bound on benchmarks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IrisBits;

impl PolyConf for IrisBits {
    const MAX_POLY_DEGREE: usize = crate::IRIS_BIT_LENGTH;
}

/// Full resolution polynomial parameters.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
#[cfg(not(tiny_poly))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FullRes;

#[cfg(not(tiny_poly))]
impl PolyConf for FullRes {
    const MAX_POLY_DEGREE: usize = 2048;
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
}
