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

#[cfg(test)]
pub mod keygen;

#[cfg(test)]
pub mod hamming;

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

    /// Sample from binary message space
    pub fn sample_binary(&self, rng: &mut ThreadRng) -> Message<C> {
        // TODO: this might be implemented more efficiently using `Rng::gen_bool()`
        let m = self.sample_uniform_range(0..=1_u64, rng);
        Message { m }
    }
}
