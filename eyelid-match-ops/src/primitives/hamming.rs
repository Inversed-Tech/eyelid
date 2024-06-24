//! Implementation of the simple encoding

use crate::primitives::poly::Poly;
use ark_ff::Zero;
use std::ops::AddAssign;

use rand::rngs::ThreadRng;

use super::yashe::Yashe;
use super::yashe::{Ciphertext, Message, PrivateKey, PublicKey, YasheConf};

/// Contains the message to be encoded such that
/// the Hamming distance can be computed later.
pub struct SimpleHammingEncoding<C: YasheConf>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// The message to be encoded
    m: Message<C>,
    /// The reverse of the message to be encoded
    m_rev: Message<C>,
}

/// SimpleHammingEncodingCiphertext is a struct that holds two ciphertexts, c and c_rev,
/// which are the encodings of the message m and m_rev, respectively. The encoding is
/// done by reversing the message and encoding it as a regular Yashe Ciphertext.
pub struct SimpleHammingEncodingCiphertext<C: YasheConf>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// The ciphertext of the message m
    c: Ciphertext<C>,
    /// The ciphertext of the message m_rev
    c_rev: Ciphertext<C>,
}

impl<C: YasheConf> SimpleHammingEncoding<C>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Creates a new `SimpleHammingEncoding` with the given message `m` and size `size`.
    pub fn new(m: Message<C>, size: usize) -> Self {
        let mut m_rev = Message {
            m: Poly::<C>::zero(),
        };
        for i in 0..size {
            m_rev.m[i] = m.m[size - i - 1];
        }
        Self { m, m_rev }
    }

    /// Sample a random SimpleHammingEncoding, by sampling a random binary Yashe Message, which
    /// is done by calling function sample_binary_message, and returning a new SimpleHammingEncoding,
    /// which sets m to the sampled message, and m_rev to the reverse of m.
    pub fn sample(ctx: Yashe<C>, size: usize, rng: &mut ThreadRng) -> SimpleHammingEncoding<C> {
        SimpleHammingEncoding::new(ctx.sample_binary_message(rng), size)
    }

    /// Compute the Hamming distance between self and v2. In order to do this,
    /// we subtract each component of the encoding, namely self.m from v2.m and self.m_rev from v2.m_rev,
    /// and multiply the obtained Messages, returning a regular Yashe Message as output.
    pub fn hamming_distance(&self, v2: SimpleHammingEncoding<C>, size: usize) -> C::Coeff {
        let res: &mut C::Coeff = &mut C::Coeff::zero();
        for i in 0..size {
            if self.m.m[i] != v2.m.m[i] {
                res.add_assign(C::Coeff::from(1u64));
            }
        }
        *res
    }

    /// Encrypts the message m encoded as a SimpleHammingEncoding, which is done by encrypting
    /// each component of the encoding separately, and returning a SimpleHammingEncodingCiphertext.
    pub fn encrypt_simple_hamming_encoding(
        &self,
        ctx: Yashe<C>,
        pub_key: &PublicKey<C>,
        rng: &mut ThreadRng,
    ) -> SimpleHammingEncodingCiphertext<C> {
        let c = ctx.encrypt(self.m.clone(), pub_key, rng);
        let c_rev = ctx.encrypt(self.m_rev.clone(), pub_key, rng);
        SimpleHammingEncodingCiphertext { c, c_rev }
    }
}

impl<C: YasheConf> SimpleHammingEncodingCiphertext<C>
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Decrypts the SimpleHammingEncodingCiphertext c, by decrypting each component of the encoding
    /// separately, and returning the result as a SimpleHammingEncoding.
    pub fn decrypt_simple_hamming_encoding(
        &self,
        ctx: Yashe<C>,
        priv_key: &PrivateKey<C>,
    ) -> SimpleHammingEncoding<C> {
        let m = ctx.decrypt(self.c.clone(), priv_key);
        let m_rev = ctx.decrypt(self.c_rev.clone(), priv_key);
        SimpleHammingEncoding { m, m_rev }
    }

    /// In order to homomorphically compute the hamming distance between two
    /// SimpleHammingEncodingCiphertexts, we need to subtract each
    /// component respectively. Namely, given c1 and c2, we need to compute
    /// a SimpleHammingEncodingCiphertext c, such that c.c = c1.c - c2.c,
    /// and c.c_rev = c1.c_rev - c2.c_rev. Then we multiply c.c by c.c_rev
    /// and return the result as a regular Yashe Ciphertext.
    pub fn homomorphic_hamming_distance(
        &self,
        ctx: Yashe<C>,
        c2: SimpleHammingEncodingCiphertext<C>,
    ) -> Ciphertext<C> {
        let c = Ciphertext {
            c: &self.c.c - &c2.c.c,
        };
        let c_rev = Ciphertext {
            c: &self.c_rev.c - &c2.c_rev.c,
        };
        ctx.ciphertext_mul(c, c_rev)
    }
}
