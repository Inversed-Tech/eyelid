//! Fq79 implemented using a single u128.

use std::{
    iter,
    num::ParseIntError,
    ops::{Add, Deref, DerefMut, Div, Mul},
    str::FromStr,
};

use ark_ff::{BigInteger, FftField, Field, PrimeField, SqrtPrecomputation};
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, Flags, SerializationError, Validate,
};
use num_bigint::BigUint;
use zeroize::Zeroize;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
/// t = 2ˆ15, q = 2ˆ79, N = 2048
//
// Sage commands:
// random_prime(2**79)
// 93309596432438992665667
// ff = GF(93309596432438992665667)
// ff.multiplicative_generator()
// 5
//
// We could also consider generating primes dynamically, but this could impact performance.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Zeroize)]
pub struct Fq79(pub u128);

impl Deref for Fq79 {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fq79 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<u128> for Fq79 {
    fn into(self) -> u128 {
        todo!()
    }
}

impl From<u128> for Fq79 {
    fn from(_value: u128) -> Self {
        todo!()
    }
}

impl Into<BigUint> for Fq79 {
    fn into(self) -> BigUint {
        todo!()
    }
}

impl From<BigUint> for Fq79 {
    fn from(_value: BigUint) -> Self {
        todo!()
    }
}

impl FromStr for Fq79 {
    type Err = ParseIntError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl CanonicalSerialize for Fq79 {
    /// The general serialize method that takes in customization flags.
    fn serialize_with_mode<W: ark_std::io::Write>(
        &self,
        _writer: W,
        _compress: Compress,
    ) -> Result<(), SerializationError> {
        todo!()
    }

    fn serialized_size(&self, _compress: Compress) -> usize {
        Self::MODULUS_BIT_SIZE as usize
    }
}

impl CanonicalDeserialize for Fq79 {
    fn deserialize_with_mode<R: ark_std::io::Read>(
        _reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        todo!()
    }
}

impl Add for Fq79 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        ((self.0 + rhs.0) % Self::MODULUS.0).into()
    }
}

impl Mul for Fq79 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        ((self.0 * rhs.0) % Self::MODULUS.0).into()
    }
}

impl BigInteger for Fq79 {
    const NUM_LIMBS: usize = 1;

    fn add_with_carry(&mut self, _other: &Self) -> bool {
        todo!()
    }

    fn sub_with_borrow(&mut self, _other: &Self) -> bool {
        todo!()
    }

    fn mul2(&mut self) -> bool {
        todo!()
    }

    fn muln(&mut self, _amt: u32) {
        todo!()
    }

    fn div2(&mut self) {
        todo!()
    }

    fn divn(&mut self, _amt: u32) {
        todo!()
    }

    fn is_odd(&self) -> bool {
        todo!()
    }

    fn is_even(&self) -> bool {
        todo!()
    }

    fn is_zero(&self) -> bool {
        todo!()
    }

    fn num_bits(&self) -> u32 {
        todo!()
    }

    fn get_bit(&self, _i: usize) -> bool {
        todo!()
    }

    fn from_bits_be(_bits: &[bool]) -> Self {
        todo!()
    }

    fn from_bits_le(_bits: &[bool]) -> Self {
        todo!()
    }

    fn to_bytes_be(&self) -> Vec<u8> {
        todo!()
    }

    fn to_bytes_le(&self) -> Vec<u8> {
        todo!()
    }
}

impl FftField for Fq79 {
    const GENERATOR: Self = Fq79(5);

    const TWO_ADICITY: u32 = {
        // Modified from:
        // https://github.com/arkworks-rs/algebra/blob/f980e761421ac22039afc47e662fa9b895e1a6f0/ff/src/biginteger/mod.rs#L189-L202
        let mut two_adicity = 0;

        let mut modulus = Self::MODULUS.0;

        assert!(modulus % 2 == 1);

        // Since modulus is odd, we can always subtract one
        modulus -= 1;

        while modulus % 2 == 0 {
            modulus >>= 1;
            two_adicity += 1;
        }

        two_adicity
    };

    //Self::GENERATOR.modpow(Self::TRACE, Self::MODULUS), but using Python:
    // >>> pow(5, 46654798216219496332833, 93309596432438992665667)
    // 93309596432438992665666L
    //
    // Based on:
    // https://github.com/rust-num/num-bigint/blob/e9b204cf5abd91dda241a921444cce4abcc6f885/src/biguint/power.rs#L149-L218
    const TWO_ADIC_ROOT_OF_UNITY: Self = Fq79(93309596432438992665666_u128);
}

impl PrimeField for Fq79 {
    type BigInt = Fq79;

    const MODULUS: Self::BigInt = Fq79(93309596432438992665667_u128);

    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt = Fq79((Self::MODULUS.0 - 1) / 2);

    const MODULUS_BIT_SIZE: u32 = u128::BITS;

    // Python commands:
    // >>> bin(93309596432438992665667 - 1)
    // '0b10011110000100101001011000111100110111101111110010110100000010111110001000010'
    // >>> (93309596432438992665667 - 1) >> 1
    // 46654798216219496332833L
    //
    // Based on:
    // https://github.com/arkworks-rs/algebra/blob/f980e761421ac22039afc47e662fa9b895e1a6f0/ff/src/biginteger/mod.rs#L204-L217
    const TRACE: Self::BigInt = Fq79(46654798216219496332833_u128);

    const TRACE_MINUS_ONE_DIV_TWO: Self::BigInt = Fq79((Self::TRACE.0 - 1) / 2);

    fn from_bigint(repr: Self::BigInt) -> Option<Self> {
        Some(repr)
    }

    fn into_bigint(self) -> Self::BigInt {
        self
    }
}

impl Field for Fq79 {
    type BasePrimeField = Self;

    type BasePrimeFieldIter = iter::Once<Self::BasePrimeField>;

    // TODO: is this important?
    const SQRT_PRECOMP: Option<SqrtPrecomputation<Self>> = None;
    /*
    {
        // Modified from:
        // https://github.com/arkworks-rs/algebra/blob/f980e761421ac22039afc47e662fa9b895e1a6f0/ff/src/fields/models/fp/montgomery_backend.rs#L543-L559
        match Self::BasePrimeField::MODULUS.mod_4() {
            3 => match <MontBackend<T, N>>::MODULUS_PLUS_ONE_DIV_FOUR.as_ref() {
                Some(BigInt(modulus_plus_one_div_four)) => Some(SqrtPrecomputation::Case3Mod4 {
                    modulus_plus_one_div_four,
                }),
                None => None,
            },
            _ => Some(SqrtPrecomputation::TonelliShanks {
                two_adicity: <MontBackend<T, N>>::TWO_ADICITY,
                quadratic_nonresidue_to_trace: T::TWO_ADIC_ROOT_OF_UNITY,
                trace_of_modulus_minus_one_div_two:
                    &<Fp<MontBackend<T, N>, N>>::TRACE_MINUS_ONE_DIV_TWO.0,
            }),
        }
    };
    */

    const ZERO: Self = Fq79(0);

    const ONE: Self = Fq79(1);

    fn extension_degree() -> u64 {
        todo!()
    }

    fn to_base_prime_field_elements(&self) -> Self::BasePrimeFieldIter {
        todo!()
    }

    fn from_base_prime_field_elems(_elems: &[Self::BasePrimeField]) -> Option<Self> {
        todo!()
    }

    fn from_base_prime_field(_elem: Self::BasePrimeField) -> Self {
        todo!()
    }

    fn double(&self) -> Self {
        todo!()
    }

    fn double_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn neg_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn from_random_bytes_with_flags<F: Flags>(_bytes: &[u8]) -> Option<(Self, F)> {
        todo!()
    }

    fn legendre(&self) -> ark_ff::LegendreSymbol {
        todo!()
    }

    fn square(&self) -> Self {
        todo!()
    }

    fn square_in_place(&mut self) -> &mut Self {
        todo!()
    }

    fn inverse(&self) -> Option<Self> {
        todo!()
    }

    fn inverse_in_place(&mut self) -> Option<&mut Self> {
        todo!()
    }

    fn frobenius_map_in_place(&mut self, _power: usize) {
        todo!()
    }
}
