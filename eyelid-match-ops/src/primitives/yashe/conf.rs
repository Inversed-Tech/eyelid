//! Fixed parameters for the YASHE encryption scheme.
//!
//! Temporarily switch to tiny parameters to make test errors easier to debug:
//! ```text
//! RUSTFLAGS="--cfg tiny_poly" cargo test
//! RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
//! ```

use ark_ff::PrimeField;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use crate::{
    primitives::poly::{modular_poly::conf::IrisBits, PolyConf},
    IRIS_BIT_LENGTH,
};

pub use crate::primitives::poly::modular_poly::conf::TestRes;

#[cfg(not(tiny_poly))]
use crate::primitives::poly::modular_poly::conf::FullRes;

#[cfg(tiny_poly)]
use crate::primitives::poly::modular_poly::conf::TinyTest;

/// Fixed YASHE encryption scheme parameters.
/// The [`PolyConf`] supertrait is the configuration of the polynomials used in the scheme.
///
/// Encryption keys and ciphertexts with different parameters are incompatible.
pub trait YasheConf: PolyConf
where
    Self::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// The plaintext coefficient modulus
    const T: u64;

    /// The standard deviation for key generation sampling.
    /// The default parameters are as recommended in the paper.
    const KEY_DELTA: f64 = 3.2;

    /// The standard deviation for encryption error sampling
    /// The default parameters are as recommended in the paper.
    const ERROR_DELTA: f64 = 1.0;

    /// A convenience method to convert [`T`](Self::T) to the [`Coeff`](PolyConf::Coeff) type.
    fn t_as_coeff() -> Self::Coeff {
        Self::Coeff::from(Self::T)
    }

    /// A convenience method to convert [`T`](Self::T) to `u128`.
    fn t_as_u128() -> u128 {
        u128::from(Self::T)
    }

    /// A convenience method to convert [`Coeff::MODULUS`](PrimeField::MODULUS) to `u128`.
    fn modulus_as_u128() -> u128 {
        let modulus: BigUint = Self::Coeff::MODULUS.into();

        modulus
            .to_u128()
            .expect("constant modulus is small enough for u128")
    }
}

/// Iris bit length polynomial parameters.
///
/// This uses the full number of iris bits, which gives an upper bound on benchmarks.
impl YasheConf for IrisBits {
    const T: u64 = IRIS_BIT_LENGTH as u64;
}

/// Full resolution polynomial parameters.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
#[cfg(not(tiny_poly))]
impl YasheConf for FullRes {
    const T: u64 = 1024;
}

/// Tiny test polynomials, used for finding edge cases in tests.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.

#[cfg(tiny_poly)]
impl YasheConf for TinyTest {
    /// Limited to the modulus of the underlying `Coeff` type.
    const T: u64 = 7;

    /// Limited to 1/6 of the modulus, so that the sampled values are valid within 6 sigmas.
    const KEY_DELTA: f64 = 0.9;

    /// Limited to 1/3 of KEY_DELTA, so that the error is small enough for valid decryption.
    const ERROR_DELTA: f64 = 0.3;
}
