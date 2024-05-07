//! Implementation of YASHE cryptosystem
//! `<https://eprint.iacr.org/2013/075.pdf>`

use std::marker::PhantomData;

use ark_ff::{BigInt, BigInteger, One, PrimeField, UniformRand};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};

use crate::primitives::poly::{Coeff, Poly, PolyConf};

#[cfg(test)]
pub mod test;

/// Yashe parameters
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct YasheParams {
    /// Plaintext coefficient modulus
    pub t: u64,
    /// Standard deviation
    pub delta: f64,
}

/// Yashe scheme
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Yashe<C: PolyConf> {
    /// Cryptosystem parameters
    /// TODO: turn these into a trait and marker type, with a `PolyConf` type in the cryptosystem trait
    params: YasheParams,

    /// A zero-sized marker, which binds the config type to the outer type.
    _conf: PhantomData<C>,
}

/// Private key struct
#[derive(Debug, Clone)]
pub struct PrivateKey<C: PolyConf> {
    /// Sampled with small coefficients (and invertible)
    pub f: Poly<C>,
    /// The inverse of f
    pub finv: Poly<C>,
    /// Private key
    pub priv_key: Poly<C>,
}

/// Public key struct
#[derive(Debug)]
pub struct PublicKey<C: PolyConf> {
    /// Public key
    pub h: Poly<C>,
}

/// Message struct
#[derive(Debug)]
pub struct Message<C: PolyConf> {
    /// Message encoded as a polynomial
    pub m: Poly<C>,
}

/// Ciphertext struct
#[derive(Debug)]
pub struct Ciphertext<C: PolyConf> {
    /// Ciphertext encoded as a polynomial
    pub c: Poly<C>,
}

impl<C: PolyConf> Yashe<C> {
    /// Yashe constructor
    pub fn new(params: YasheParams) -> Self {
        Self {
            params,
            _conf: PhantomData,
        }
    }

    /// Generate the private key
    pub fn generate_private_key(&self, rng: &mut ThreadRng) -> PrivateKey<C> {
        loop {
            let f = self.sample_key(rng);
            let finv = f.inverse();

            let Ok(finv) = finv else {
                continue;
            };

            let mut priv_key = f.clone();
            priv_key *= Coeff::from(self.params.t);
            priv_key[0] += Coeff::one();
            priv_key.truncate_to_canonical_form();

            let priv_key_inv = priv_key.inverse();

            if let Ok(_priv_key_inv) = priv_key_inv {
                return PrivateKey { f, finv, priv_key };
            }
        }
    }

    /// Generate the public key
    pub fn generate_public_key(
        &self,
        rng: &mut ThreadRng,
        private_key: &PrivateKey<C>,
    ) -> PublicKey<C> {
        let mut h = self.sample_uniform(rng);

        h *= Coeff::from(self.params.t);
        h.truncate_to_canonical_form();
        h = h * &private_key.finv;

        PublicKey { h }
    }

    /// Generate the key pair
    pub fn keygen(&self, rng: &mut ThreadRng) -> (PrivateKey<C>, PublicKey<C>) {
        let priv_key = self.generate_private_key(rng);
        let pub_key = self.generate_public_key(rng, &priv_key);
        (priv_key, pub_key)
    }

    pub fn encrypt(&self, m: Message<C>, public_key: PublicKey<C>, rng: &mut ThreadRng) -> Ciphertext<C> {
        let logt = self.params.t.ilog2();
        let s = self.sample_err(rng);
        let e = self.sample_err(rng);
        let mut c = s * public_key.h + e;
        // TODO: find a way to access q and divide by t
        let mut qdt: BigInt<2> = BigInt::from(Coeff::MODULUS);
        //let qbt = q / Coeff::from(self.params.t);
        qdt.divn(logt);
        let mqdt = m.m * Coeff::from(qdt);
        c = c + mqdt;
        Ciphertext { c }
    }

    pub fn decrypt(&self, c: Ciphertext<C>, private_key: PrivateKey<C>) -> Message<C> {
        let logt = self.params.t.ilog2();
        let q = Coeff::MODULUS;
        let qm1d2 = Coeff::MODULUS_MINUS_ONE_DIV_TWO;
        let mut res = c.c * private_key.f;
        for i in 0..C::MAX_POLY_DEGREE {
            // Convert coefficient to big integer
            let mut coeff_res = res[i].into_bigint();
            // Multiply by t
            coeff_res.muln(logt);
            if coeff_res >= BigInt::from(0) {
                // TODO: deal with carry
                coeff_res.add_with_carry(&qm1d2);
                // TODO: how to divide 2 bigints?
                coeff_res = coeff_res / q;
            } else {
                // TODO: deal with borrow
                coeff_res.sub_with_borrow(&qm1d2);
                // TODO: how to divide 2 bigints?
                coeff_res = coeff_res / q;
            }
            // TODO: validate the following
            // REMARK: since t is a power of 2, we have that coeff_res.0[i] is zero for i > 0
            // then we can apply the following formula
            let result = coeff_res.0[0] % self.params.t;
            res[i] = result.into();
        }
        Message { m: res }
    }

    /// Sample a polynomial with small random coefficients using a gaussian distribution.
    #[allow(clippy::cast_possible_truncation)]
    pub fn sample_err(&self, rng: &mut ThreadRng) -> Poly<C> {
        self.sample_gaussian(self.params.delta, rng)
    }

    /// Sample a polynomial with small random coefficients using a gaussian distribution.
    #[allow(clippy::cast_possible_truncation)]
    pub fn sample_key(&self, rng: &mut ThreadRng) -> Poly<C> {
        // standard deviation whose output coefficients are -1, 0, 1 with high probability
        self.sample_gaussian(self.params.delta/8.0, rng)
    }

    /// Sample a polynomial with small random coefficients using a gaussian distribution.
    #[allow(clippy::cast_possible_truncation)]
    pub fn sample_gaussian(&self, delta: f64, rng: &mut ThreadRng) -> Poly<C> {
        let mut res = Poly::non_canonical_zeroes(C::MAX_POLY_DEGREE);
        for i in 0..C::MAX_POLY_DEGREE {
            // TODO SECURITY: check that the generated integers are secure:
            // <https://github.com/Inversed-Tech/eyelid/issues/70>
            let normal = Normal::new(0.0, delta).unwrap();
            let v: f64 = normal.sample(rng);

            // TODO: try i128, i32, i16, or i8 here
            //
            // Until we've checked the security of using fewer bits, use a large and performant type.
            // Larger values are extremely rare, and will saturate to MIN or MAX.
            // This is ok because the Coeff modulus is smaller than MIN/MAX.
            res[i] = Coeff::from(v as i64);
        }
        res.truncate_to_canonical_form();
        res
    }

    /// Sample a polynomial with unlimited size random coefficients using a uniform distribution.
    pub fn sample_uniform(&self, mut rng: &mut ThreadRng) -> Poly<C> {
        let mut res = Poly::non_canonical_zeroes(C::MAX_POLY_DEGREE);
        for i in 0..C::MAX_POLY_DEGREE {
            let coeff_rand = Coeff::rand(&mut rng);
            res[i] = coeff_rand;
        }
        res.truncate_to_canonical_form();
        res
    }
}
