//! Implementation of YASHE cryptosystem
//! `<https://eprint.iacr.org/2013/075.pdf>`

use crate::primitives::poly::{inverse, modular_poly::Poly, Coeff};
use ark_ff::{One, UniformRand};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};

#[cfg(test)]
pub mod test;

/// Yashe parameters
#[derive(Clone)]
pub struct YasheParams {
    /// Plaintext coefficient modulus
    pub t: u64,
    /// Standard deviation
    pub delta: f64,
}

/// Yashe scheme
pub struct Yashe<const MAX_POLY_DEGREE: usize> {
    /// Cryptosystem parameters
    params: YasheParams,
}

/// Private key struct
#[derive(Debug, Clone)]
pub struct PrivateKey<const MAX_POLY_DEGREE: usize> {
    /// Sampled with small coefficients (and invertible)
    pub f: Poly<MAX_POLY_DEGREE>,
    /// The inverse of f
    pub finv: Poly<MAX_POLY_DEGREE>,
    /// Private key
    pub priv_key: Poly<MAX_POLY_DEGREE>,
}

/// Public key struct
#[derive(Debug)]
pub struct PublicKey<const MAX_POLY_DEGREE: usize> {
    /// Public key
    pub h: Poly<MAX_POLY_DEGREE>,
}

impl<const MAX_POLY_DEGREE: usize> Yashe<MAX_POLY_DEGREE> {
    /// Yashe constructor
    pub fn new(params: YasheParams) -> Self {
        Self { params }
    }

    /// Generate the private key
    pub fn generate_private_key(&self, rng: ThreadRng) -> PrivateKey<MAX_POLY_DEGREE> {
        loop {
            let f = self.sample_gaussian(rng.clone());
            let mut priv_key = f.clone();
            let finv = inverse(&f);

            priv_key *= Coeff::from(self.params.t);
            priv_key[0] += Coeff::one();
            priv_key.truncate_to_canonical_form();

            if let Ok(finv) = finv {
                return PrivateKey { f, finv, priv_key };
            }
        }
    }

    /// Generate the public key
    pub fn generate_public_key(
        &self,
        rng: ThreadRng,
        private_key: PrivateKey<MAX_POLY_DEGREE>,
    ) -> PublicKey<MAX_POLY_DEGREE> {
        let q = self.sample_rand(rng);
        let mut h = q.clone();
        h *= Coeff::from(self.params.t);
        h.truncate_to_canonical_form();
        h = h * &private_key.finv;
        PublicKey { h }
    }

    /// Generate the key pair
    pub fn keygen(
        &self,
        rng: ThreadRng,
    ) -> (PrivateKey<MAX_POLY_DEGREE>, PublicKey<MAX_POLY_DEGREE>) {
        let priv_key = self.generate_private_key(rng.clone());
        let pub_key = self.generate_public_key(rng, priv_key.clone());
        (priv_key, pub_key)
    }

    /// This sampling is similar to what will be necessary for YASHE KeyGen.
    /// The purpose is to obtain a polynomial with small random coefficients.
    #[allow(clippy::cast_possible_truncation)]
    pub fn sample_gaussian(&self, mut rng: ThreadRng) -> Poly<MAX_POLY_DEGREE> {
        let mut res = Poly::non_canonical_zeroes(MAX_POLY_DEGREE);
        for i in 0..MAX_POLY_DEGREE {
            let normal = Normal::new(0.0, self.params.delta).unwrap();
            let v: f64 = normal.sample(&mut rng);
            res[i] = Coeff::from(v as i64);
        }
        res.truncate_to_canonical_form();
        res
    }

    /// This sampling is similar to what will be necessary for YASHE KeyGen.
    /// The purpose is to obtain a polynomial with small random coefficients.
    pub fn sample_rand(&self, mut rng: ThreadRng) -> Poly<MAX_POLY_DEGREE> {
        let mut res = Poly::non_canonical_zeroes(MAX_POLY_DEGREE);
        for i in 0..MAX_POLY_DEGREE {
            let coeff_rand = Coeff::rand(&mut rng);
            res[i] = coeff_rand;
        }
        res.truncate_to_canonical_form();
        res
    }
}
