//! Fixed parameters for the YASHE encryption scheme.
//!
//! Temporarily switch to tiny parameters to make test errors easier to debug:
//! ```text
//! RUSTFLAGS="--cfg tiny_poly" cargo test
//! RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
//! ```

use crate::{iris::conf::IrisConf, primitives::poly::PolyConf, FullRes, IrisBits};

#[cfg(tiny_poly)]
use crate::TinyTest;

/// Fixed YASHE encryption scheme parameters.
/// The [`PolyConf`] supertrait is the configuration of the polynomials used in the scheme.
///
/// Encryption keys and ciphertexts with different parameters are incompatible.
pub trait YasheConf: PolyConf
where
    Self::Coeff: From<u64> + From<i64>,
{
    /// The plaintext coefficient modulus
    const T: u64;

    /// The standard deviation
    const DELTA: f64;

    /// A convenience method to convert `T` to the `Coeff` type.
    fn t_as_coeff() -> Self::Coeff {
        Self::Coeff::from(Self::T)
    }
}

/// Iris bit length polynomial parameters.
///
/// This uses the full number of iris bits, which gives an upper bound on benchmarks.
impl YasheConf for IrisBits {
    const T: u64 = Self::BIT_LENGTH as u64;

    const DELTA: f64 = 3.2;
}
// TODO: work out how to constant assert these constraints for each config type:
// The encrypted coefficient modulus can't be smaller than the plaintext modulus.
// const_assert!(IrisBits::T <= <IrisBits as PolyConf>::Coeff::MODULUS as u128);
// The standard deviation must fit within the encrypted coefficient modulus, with six sigma probability.
// const_assert!(IrisBits::DELTA <= <IrisBits as PolyConf>::Coeff::MODULUS as f64 / 6.0);

/// Full resolution polynomial parameters.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
impl YasheConf for FullRes {
    const T: u64 = 1024;

    const DELTA: f64 = 3.2;
}

/// Tiny test polynomials, used for finding edge cases in tests.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.

#[cfg(tiny_poly)]
impl YasheConf for TinyTest {
    /// Limited to the modulus of the underlying `Coeff` type.
    const T: u64 = 7;

    /// Limited to 1/6 of the modulus, so that the sampled values are valid within 6 sigmas.
    const DELTA: f64 = 0.9;
}
