use crate::primitives::poly::{inverse, modular_poly::Poly, sample_gaussian, sample_rand, Coeff};
use ark_ff::One;
use rand::rngs::ThreadRng;

pub mod test;

/// Yashe parameters
pub struct YasheParams {
    t: u64,
    _delta: f64,
    // TODO: include q and n
}

/// Yashe scheme
pub struct Yashe<const MAX_POLY_DEGREE: usize> {
    params: YasheParams,
}

/// Private key struct
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PrivateKey<const MAX_POLY_DEGREE: usize> {
    f: Poly<MAX_POLY_DEGREE>,
    finv: Poly<MAX_POLY_DEGREE>,
    priv_key: Poly<MAX_POLY_DEGREE>,
}

/// Public key struct
#[derive(Debug)]
#[allow(dead_code)]
pub struct PublicKey<const MAX_POLY_DEGREE: usize> {
    h: Poly<MAX_POLY_DEGREE>,
}

impl<const MAX_POLY_DEGREE: usize> Yashe<MAX_POLY_DEGREE> {
    /// Yashe constructor
    pub fn new(params: YasheParams) -> Self {
        Self { params }
    }

    /// Generate the private key
    pub fn generate_private_key(&self, rng: ThreadRng) -> PrivateKey<MAX_POLY_DEGREE> {
        loop {
            let f = sample_gaussian::<MAX_POLY_DEGREE>(rng.clone());
            let mut priv_key = f.clone();
            let finv = inverse(&f);

            // TODO: use function to multiply by a constant
            for i in 0..MAX_POLY_DEGREE {
                priv_key[i] *= Coeff::from(self.params.t);
            }
            priv_key[0] += Coeff::one();
            priv_key.truncate_to_canonical_form();

            if finv.is_ok() {
                let finv = finv.unwrap();
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
        let q = sample_rand::<MAX_POLY_DEGREE>(rng);
        let mut h = q.clone();
        // TODO: use function to multiply by a constant
        for i in 0..MAX_POLY_DEGREE {
            h[i] *= Coeff::from(self.params.t);
        }
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
}
