//! Fixed parameters for the YASHE encryption scheme.

use std::fmt::Debug;

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

/// Fixed YASHE encryption scheme parameters.
///
/// Encryption keys and ciphertexts with different parameters are incompatible.
pub trait YasheConf: Copy + Clone + Debug + Eq + PartialEq {
    /// The configuration of the polynomials used in the scheme.
    type Poly: PolyConf;

    /// The plaintext coefficient modulus
    const T: u64;

    /// The standard deviation
    const DELTA: f64;
}

/// Iris bit length polynomial parameters.
///
/// This uses the full number of iris bits, which gives an upper bound on benchmarks.
impl YasheConf for IrisBits {
    type Poly = Self;

    const T: u64 = IRIS_BIT_LENGTH as u64;

    const DELTA: f64 = 3.2;
}

/// Full resolution polynomial parameters.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
#[cfg(not(tiny_poly))]
impl YasheConf for FullRes {
    type Poly = Self;

    const T: u64 = 1024 as u64;

    const DELTA: f64 = 3.2;
}

/// Tiny test polynomials, used for finding edge cases in tests.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.

#[cfg(tiny_poly)]
impl YasheConf for TinyTest {
    type Poly = Self;

    /// Limited to the modulus of the underlying `Coeff` type.
    const T: u64 = 7 as u64;

    /// Limited to 1/6 of the modulus, so that the sampled values are valid within 6 sigmas.
    const DELTA: f64 = 0.9;
}
