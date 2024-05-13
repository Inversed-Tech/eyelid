//! Implementation of YASHE cryptosystem
//! `<https://eprint.iacr.org/2013/075.pdf>`

use rand::Rng;
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
    pub priv_key_inv: Poly<C>,
    /// Private key
    pub priv_key: Poly<C>,
}

/// Public key struct
#[derive(Debug, Clone)]
pub struct PublicKey<C: PolyConf> {
    /// Public key
    pub h: Poly<C>,
}

/// Message struct
#[derive(Debug, Clone)]
pub struct Message<C: PolyConf> {
    /// Message encoded as a polynomial
    pub m: Poly<C>,
}

/// Ciphertext struct
#[derive(Debug, Clone)]
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

            let mut priv_key = f.clone();
            priv_key *= Coeff::from(self.params.t);
            priv_key[0] += Coeff::one();
            priv_key.truncate_to_canonical_form();

            let priv_key_inv = priv_key.inverse();

            if let Ok(priv_key_inv) = priv_key_inv {
                return PrivateKey {
                    f,
                    priv_key_inv,
                    priv_key,
                };
            }
        }
    }

    /// Generate the public key
    pub fn generate_public_key(
        &self,
        rng: &mut ThreadRng,
        private_key: &PrivateKey<C>,
    ) -> PublicKey<C> {
        let mut h = self.sample_key(rng);

        h *= Coeff::from(self.params.t);
        h.truncate_to_canonical_form();
        h = h * &private_key.priv_key_inv;

        PublicKey { h }
    }

    /// Generate the key pair
    pub fn keygen(&self, rng: &mut ThreadRng) -> (PrivateKey<C>, PublicKey<C>) {
        let priv_key = self.generate_private_key(rng);
        let pub_key = self.generate_public_key(rng, &priv_key);
        (priv_key, pub_key)
    }

    /// Encrypt a message m encoded in the polynomial ring
    pub fn encrypt(
        &self,
        m: Message<C>,
        public_key: PublicKey<C>,
        rng: &mut ThreadRng,
    ) -> Ciphertext<C> {
        let logt = self.params.t.ilog2();
        let s = self.sample_err(rng);
        let e = self.sample_err(rng);
        let mut c = s * public_key.h + e;

        let mut qdt: BigInt<2> = Coeff::MODULUS;
        qdt.divn(logt);
        let mqdt = m.m * Coeff::from(qdt);
        c += mqdt;
        c.truncate_to_canonical_form();
        Ciphertext { c }
    }

    /// Convert bigint to u128
    // TODO: implement u128::from
    pub fn convert_to_u128(&self, a: BigInt<2>) -> u128 {
        let mut res: u128 = u128::from(a.0[0]);
        res += u128::from(a.0[1]) * 2u128.pow(64);
        res
    }

    /// Decrypt a ciphertext
    pub fn decrypt(&self, c: Ciphertext<C>, private_key: PrivateKey<C>) -> Message<C> {
        let logt = self.params.t.ilog2();
        let q = Coeff::MODULUS;
        let qm1d2 = Coeff::MODULUS_MINUS_ONE_DIV_TWO;
        let mut res = c.c * private_key.priv_key;
        for i in 0..C::MAX_POLY_DEGREE {
            // Convert coefficient to big integer
            let mut coeff_res = res[i].into_bigint();
            // Multiply by t
            coeff_res.muln(logt);
            let carry = coeff_res.add_with_carry(&qm1d2);
            if carry {
                panic!("carry should not happen");
            }
            let coeff_res_u128 = self.convert_to_u128(coeff_res) / self.convert_to_u128(q);

            let result = coeff_res_u128 % u128::from(self.params.t);
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
    /// TODO: this function seems to be returning too few non-zero elements
    #[allow(clippy::cast_possible_truncation)]
    pub fn sample_key(&self, rng: &mut ThreadRng) -> Poly<C> {
        // standard deviation whose output coefficients are -1, 0, 1 with high probability
        self.sample_gaussian(self.params.delta / 8.0, rng)
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

    /// Sample from message space
    pub fn sample_message(&self, rng: &mut ThreadRng) -> Message<C> {
        let mut res = Poly::non_canonical_zeroes(C::MAX_POLY_DEGREE);
        for i in 0..C::MAX_POLY_DEGREE {
            let coeff_rand: u64 = rng.gen_range(0..self.params.t);
            res[i] = coeff_rand.into();
        }
        res.truncate_to_canonical_form();
        Message { m: res }
    }

    /// Sample from binary message space
    pub fn sample_binary_message(&self, rng: &mut ThreadRng) -> Message<C> {
        let mut res = Poly::non_canonical_zeroes(C::MAX_POLY_DEGREE);
        for i in 0..C::MAX_POLY_DEGREE {
            let coeff_rand: u64 = rng.gen_range(0..2);
            res[i] = coeff_rand.into();
        }
        res.truncate_to_canonical_form();
        Message { m: res }
    }
}
