//! Tests for YASHE cryptosystem.

use ark_ff::{One, Zero};
use rand::rngs::ThreadRng;

use crate::primitives::{
    poly::Poly,
    yashe::{Message, Yashe, YasheConf},
};

#[cfg(test)]
pub mod encdec;

#[cfg(test)]
pub mod hom;

mod hamming;
#[cfg(test)]
pub mod keygen;

// Test-only data generation methods.
impl<C: YasheConf> Yashe<C>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Sample from message space
    pub fn sample_message(&self, rng: &mut ThreadRng) -> Message<C> {
        let m = self.sample_uniform_range(0..C::T, rng);
        Message { m }
    }

    /// "Sample" one
    pub fn sample_one(&self) -> Message<C> {
        let mut m = Poly::<C>::zero();
        m[0] = C::Coeff::one();
        Message { m }
    }

    /// "Sample" constant
    pub fn sample_constant(&self, c: u64) -> Message<C> {
        let mut m = Poly::<C>::zero();
        m[0] = C::Coeff::from(c);
        Message { m }
    }

    /// "Sample" zero
    pub fn sample_zero(&self) -> Message<C> {
        let m = Poly::<C>::zero();
        Message { m }
    }
}
