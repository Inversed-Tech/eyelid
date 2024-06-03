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

use crate::{primitives::poly::PolyConf, FullRes, IrisBits, MiddleRes};

#[cfg(tiny_poly)]
use crate::TinyTest;

/// Fixed YASHE encryption scheme parameters.
/// The [`PolyConf`] supertrait is the configuration of the polynomials used in the scheme.
///
/// Encryption keys and ciphertexts with different parameters are incompatible.
pub trait YasheConf: PolyConf
where
    // The `Field` trait is already `From<u128> + From<u64>` (and all the other unsigned types).
    // The `Fp` types are `From<i64>` (and all the other signed types).
    // But there are no trait bounds guaranteeing these conversions, so we need to require them.
    // Unfortunately, these bounds also need to be copied to each generic type and impl block.
    Self::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// The plaintext coefficient modulus.
    /// Must be a power of two, and smaller than the modulus.
    const T: u64;

    /// The standard deviation for key generation sampling.
    /// The default parameters are as recommended in the paper.
    const KEY_DELTA: f64 = 3.2;

    /// The standard deviation for encryption error sampling
    /// The default parameters are as recommended in the paper.
    const ERROR_DELTA: f64 = 1.0;

    /// A convenience method to convert [`T`](Self::T) to the [`Coeff`](PolyConf::Coeff) type.
    fn t_as_coeff() -> Self::Coeff {
        debug_assert!(check_constraints::<Self>());

        Self::Coeff::from(Self::T)
    }

    /// A convenience method to convert [`T`](Self::T) to `u128`.
    // The u64 to u128 cast is checked for type changes by `check_constraints()`.
    #[allow(clippy::cast_lossless)]
    fn t_as_u128() -> u128 {
        Self::T as u128
    }

    /// A convenience method to convert [`T`](Self::T) to `i128`.
    fn t_as_i128() -> i128 {
        i128::from(Self::T)
    }

    /// A convenience method to convert a [`Coeff`](PolyConf::Coeff) to `u128`.
    /// TODO: move this method to a trait implemented on `Coeff` instead.
    fn coeff_as_u128(coeff: Self::Coeff) -> u128 {
        let coeff: BigUint = coeff.into();

        coeff
            .to_u128()
            .expect("coefficients are small enough for u128")
    }

    /// A convenience method to convert a [`Coeff`](PolyConf::Coeff) to `i128`.
    /// TODO: move this method to a trait implemented on `Coeff` instead.
    fn coeff_as_i128(coeff: Self::Coeff) -> i128 {
        let coeff: BigUint = coeff.into();

        coeff
            .to_i128()
            .expect("coefficients are small enough for i128")
    }

    /// A convenience method to convert [`Coeff::MODULUS`](PrimeField::MODULUS) to `u128`.
    fn modulus_as_u128() -> u128 {
        // We can't check constraints here, because this method is called by the constraint checks.

        let modulus: BigUint = Self::Coeff::MODULUS.into();

        modulus
            .to_u128()
            .expect("constant modulus is small enough for u128")
    }

    /// A convenience method to convert [`Coeff::MODULUS`](PrimeField::MODULUS) to `i128`.
    fn modulus_as_i128() -> i128 {
        let modulus: BigUint = Self::Coeff::MODULUS.into();

        modulus
            .to_i128()
            .expect("constant modulus is small enough for i128")
    }

    /// A convenience method to convert [`Coeff::MODULUS_MINUS_ONE_DIV_TWO`](PrimeField::MODULUS_MINUS_ONE_DIV_TWO) to `u128`.
    fn modulus_minus_one_div_two_as_u128() -> u128 {
        let modulus: BigUint = Self::Coeff::MODULUS_MINUS_ONE_DIV_TWO.into();

        modulus
            .to_u128()
            .expect("constant modulus is small enough for u128")
    }

    /// A convenience method to convert [`Coeff::MODULUS_MINUS_ONE_DIV_TWO`](PrimeField::MODULUS_MINUS_ONE_DIV_TWO) to `i128`.
    fn modulus_minus_one_div_two_as_i128() -> i128 {
        let modulus: BigUint = Self::Coeff::MODULUS_MINUS_ONE_DIV_TWO.into();

        modulus
            .to_i128()
            .expect("constant modulus is small enough for i128")
    }
}

/// Checks various constraints on the generic values.
//
// The u64 to f64 cast keeps precision because the values are all small compared to the types.
// There is an assertion that checks this remains valid, even if the types or values change.
#[allow(clippy::cast_precision_loss)]
// The u64 to u128 cast is checked for type changes in the const check.
#[allow(clippy::cast_lossless)]
fn check_constraints<C: YasheConf>() -> bool
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    let () = Assert::<C>::CHECK;

    // The encrypted coefficient modulus must be larger than the plaintext modulus.
    // `From::from()` isn't a const function, so we can't do a static assertion using it.
    //
    // TODO: work out how to const_assert!() this constraint.
    debug_assert!((C::T as u128) < C::modulus_as_u128());

    // Check that conversion from T to u128 is infallible.
    // This will hopefully get optimised out, even in debug builds.
    let _ = u128::from(C::T);

    // This return value lets us skip calling the assertions entirely in release builds.
    true
}

/// Call `Assert::<C>::CHECK` in one `YasheConf` method to check constant constraints on `YasheConf`.
///
/// Based on `static_assert_generic::static_assert!()`, but with the correct generic constraints:
/// <https://docs.rs/static_assert_generic/0.1.0/static_assert_generic/macro.static_assert.html>
struct Assert<D>
where
    D: YasheConf,
    D::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// A marker trait that binds the D generic to this struct.
    _p: core::marker::PhantomData<D>,
}

impl<D> Assert<D>
where
    D: YasheConf,
    D::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// The implementation of the constant check.
    //
    // The u64 to f64 cast keeps precision because the values are all small compared to the types.
    // There is an assertion that checks this remains valid, even if the types or values change.
    #[allow(unused)]
    #[allow(clippy::cast_precision_loss)]
    const CHECK: () = if (
        // The key standard deviation must fit within the plaintext modulus, with six sigma probability.
        // We use strictly less for floatong point assertions, because floating point equality sometimes
        // fails due to internal floating point inaccuracy, and this can vary by platform.
        D::KEY_DELTA > (D::T as f64) / 6.0 ||
        // Check the cast above remains valid.
        D::T >= (1 << f64::MANTISSA_DIGITS) ||
        // The error must be small enough to allow successful message retrieval, with three sigma probability.
        D::ERROR_DELTA > D::KEY_DELTA / 3.0
    ) {
        panic!("YasheConf parameters are invalid")
    };
}

/// Iris bit length polynomial parameters.
///
/// This uses the full number of iris bits, which gives an upper bound on benchmarks.
impl YasheConf for IrisBits {
    const T: u64 = 2048;
}

/// Full resolution polynomial parameters.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
impl YasheConf for FullRes {
    // VERIFY: max T should be 2^15, not 2^10
    const T: u64 = 1024;
}

/// Middle resolution polynomial parameters.
///
/// These are the parameters for middle resolution, according to the Inversed Tech report.
impl YasheConf for MiddleRes {
    // VERIFY: max T should be 2^12, not 2^8
    const T: u64 = 256;
}

/// Tiny test polynomials, used for finding edge cases in tests.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.
/// TODO: these parameters don't work for encryption and decryption, find some that do.
#[cfg(tiny_poly)]
impl YasheConf for TinyTest {
    /// Limited to the modulus of the underlying `Coeff` type.
    const T: u64 = 4;

    /// Limited to 1/6 of the modulus, so that the sampled values are valid within 6 sigmas.
    const KEY_DELTA: f64 = 0.6;

    /// Limited to 1/3 of KEY_DELTA, so that the error is small enough for valid decryption.
    /// This makes each error term zero with 2.5 sigma probability, and the entire error zero with 95% probability.
    const ERROR_DELTA: f64 = 0.19;
}
