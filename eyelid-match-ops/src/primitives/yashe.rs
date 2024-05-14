//! Implementation of YASHE cryptosystem
//! `<https://eprint.iacr.org/2013/075.pdf>`

use std::marker::PhantomData;

use ark_ff::{One, UniformRand};
use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    rngs::ThreadRng,
    Rng,
};
use rand_distr::{Distribution, Normal};

use crate::primitives::poly::Poly;

pub use conf::YasheConf;

pub mod conf;

#[cfg(test)]
pub mod test;

/// Yashe scheme
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Yashe<C: YasheConf>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// A zero-sized marker, which binds the config type to the outer type.
    _conf: PhantomData<C>,
}

/// Private key struct
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateKey<C: YasheConf>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Sampled with small coefficients (and invertible)
    pub f: Poly<C>,
    /// The inverse of f
    pub priv_key_inv: Poly<C>,
    /// Private key
    pub priv_key: Poly<C>,
}

/// Public key struct
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicKey<C: YasheConf>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Public key
    pub h: Poly<C>,
}

/// Message struct
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message<C: YasheConf>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Message encoded as a polynomial
    pub m: Poly<C>,
}

/// Ciphertext struct
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ciphertext<C: YasheConf>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Ciphertext encoded as a polynomial
    pub c: Poly<C>,
}

impl<C: YasheConf> Yashe<C>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Yashe constructor
    pub fn new() -> Self {
        Self { _conf: PhantomData }
    }

    /// Generate the private key
    pub fn generate_private_key(&self, rng: &mut ThreadRng) -> PrivateKey<C> {
        loop {
            let f = self.sample_key(rng);

            // TODO: document the equation that is being implemented here
            let mut priv_key = f.clone();
            priv_key *= C::t_as_coeff();

            // Raw coefficient access must be followed by a truncation check.
            priv_key[0] += C::Coeff::one();
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

        // TODO: document the equation that is being implemented here
        h *= C::t_as_coeff();
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
        mut m: Message<C>,
        public_key: PublicKey<C>,
        rng: &mut ThreadRng,
    ) -> Ciphertext<C> {
        // TODO: document the equations that are being implemented by each block.

        // Create the ciphertext by sampling error polynomials and applying them to the public key.
        let s = self.sample_err(rng);
        let e = self.sample_err(rng);
        let mut c = s * public_key.h + e;

        // Divide the polynomial coefficient modulus by T, using primitive integer arithmetic.
        let qdt = C::modulus_as_u128() / C::t_as_u128();
        let qdt = C::Coeff::from(qdt);

        // Multiply the message by the qdt scalar, and add it to the ciphertext.
        m.m *= qdt;
        c += m.m;

        Ciphertext { c }
    }

    /// Decrypt a ciphertext
    pub fn decrypt(&self, c: Ciphertext<C>, private_key: PrivateKey<C>) -> Message<C> {
        // TODO: document the equations that are being implemented by each block.

        // Multiply the ciphertext by the private key polynomial.
        let mut res = c.c * private_key.priv_key;

        // TODO: is this the equation that is being implemented here?
        // In primitive integer arithmetic, calculate:
        // (res[i] * T + (Q - 1)/2) / Q % T

        // Since this equation always results in zero for a zero coefficient, we don't need to
        // calculate leading zero terms.
        //
        // TODO:
        // consider creating Poly methods which take a closure to update each coefficient
        // - for leading zero coefficients to MAX_POLY_DEGREE, and only non-zero coeffs_mut()
        // - for Coeff and u128 arithmetic
        for coeff in res.coeffs_mut() {
            // Convert coefficient to a primitive integer
            let mut coeff_res = C::coeff_as_u128(*coeff);

            // Multiply by T
            coeff_res *= C::t_as_u128();
            // Add (Q - 1)/2 to implement rounding rather than truncation
            coeff_res += C::modulus_minus_one_div_two_as_u128();
            // Divide by Q
            coeff_res /= C::modulus_as_u128();
            // Modulo T
            coeff_res %= C::t_as_u128();

            // And update the coefficient
            *coeff = coeff_res.into();
        }

        // Raw coefficient access must be followed by a truncation check.
        res.truncate_to_canonical_form();

        Message { m: res }
    }

    /// Sample a polynomial with small random coefficients using a gaussian distribution.
    pub fn sample_err(&self, rng: &mut ThreadRng) -> Poly<C> {
        self.sample_gaussian(C::ERROR_DELTA, rng)
    }

    /// Sample a polynomial with small random coefficients using a gaussian distribution.
    /// TODO: this function seems to be returning too few non-zero elements
    pub fn sample_key(&self, rng: &mut ThreadRng) -> Poly<C> {
        // standard deviation whose output coefficients are -1, 0, 1 with high probability
        self.sample_gaussian(C::KEY_DELTA, rng)
    }

    /// Sample a polynomial with small random coefficients using a gaussian distribution.
    #[allow(clippy::cast_possible_truncation)]
    fn sample_gaussian(&self, delta: f64, rng: &mut ThreadRng) -> Poly<C> {
        let mut res = Poly::non_canonical_zeroes(C::MAX_POLY_DEGREE);

        for i in 0..C::MAX_POLY_DEGREE {
            // TODO SECURITY: check that the generated integers are secure:
            // <https://github.com/Inversed-Tech/eyelid/issues/70>
            let normal = Normal::new(0.0, delta).expect("constant parameters are valid");
            let v: f64 = normal.sample(rng);

            // TODO: try i128, i32, i16, or i8 here
            //
            // Until we've checked the security of using fewer bits, use a large and performant type.
            // Larger values are extremely rare, and will saturate to MIN or MAX.
            // This is ok because the C::Coeff modulus is smaller than MIN/MAX.
            res[i] = C::Coeff::from(v as i64);
        }

        // Raw coefficient access must be followed by a truncation check.
        res.truncate_to_canonical_form();

        res
    }

    /// Sample a polynomial with unlimited size random coefficients using a uniform distribution.
    pub fn sample_uniform_coeff(&self, mut rng: &mut ThreadRng) -> Poly<C> {
        let mut res = Poly::non_canonical_zeroes(C::MAX_POLY_DEGREE);
        for i in 0..C::MAX_POLY_DEGREE {
            let coeff_rand = C::Coeff::rand(&mut rng);
            res[i] = coeff_rand;
        }

        // Raw coefficient access must be followed by a truncation check.
        res.truncate_to_canonical_form();
        res
    }

    /// Sample from message space
    pub fn sample_message(&self, rng: &mut ThreadRng) -> Message<C> {
        let m = self.sample_uniform_range(0..C::T, rng);
        Message { m }
    }

    /// Sample from binary message space
    pub fn sample_binary_message(&self, rng: &mut ThreadRng) -> Message<C> {
        // TODO: this might be implemented more efficiently using `Rng::gen_bool()`
        let m = self.sample_uniform_range(0..=1_u64, rng);
        Message { m }
    }

    /// Sample a polynomial with random coefficients in `range` using a uniform distribution.
    fn sample_uniform_range<T, R>(&self, range: R, rng: &mut ThreadRng) -> Poly<C>
    where
        T: SampleUniform,
        R: SampleRange<T> + Clone,
        C::Coeff: From<T>,
    {
        let mut res = Poly::non_canonical_zeroes(C::MAX_POLY_DEGREE);
        for i in 0..C::MAX_POLY_DEGREE {
            let coeff_rand = rng.gen_range(range.clone());
            res[i] = coeff_rand.into();
        }

        // Raw coefficient access must be followed by a truncation check.
        res.truncate_to_canonical_form();
        res
    }
}
