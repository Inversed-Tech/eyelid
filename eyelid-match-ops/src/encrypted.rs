//! Iris matching operations on fully homomorphic encrypted, polynomial-encoded bit vectors.

use itertools::Itertools;
use num_bigint::{BigInt, BigUint};
use rand::rngs::ThreadRng;

use crate::{
    encoded::{MatchError, PolyCode, PolyQuery}, primitives::{
        yashe::{Ciphertext, Message, PrivateKey, PublicKey, Yashe},
    }, EncodeConf, PolyConf, YasheConf
};
use crate::iris::conf::IrisConf;

pub mod test;

/// An encrypted iris code, encoded in polynomials. To be stored in the database.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncryptedPolyCode<C: EncodeConf>
where
    C::PlainConf: YasheConf,
    <C::PlainConf as PolyConf>::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// The polynomials, encoding data, one block of rows each. Storage variant.
    data: Vec<Ciphertext<C::PlainConf>>,
    /// The mask polynomials.
    masks: Vec<Ciphertext<C::PlainConf>>,
}

/// An encrypted iris code, encoded in polynomials. To be matched against EncryptedPolyCode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncryptedPolyQuery<C: EncodeConf>
where
    C::PlainConf: YasheConf,
    <C::PlainConf as PolyConf>::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// The polynomials, encoding data, one block of rows each. Query variant.
    data: Vec<Ciphertext<C::PlainConf>>,
    /// The mask polynomials.
    masks: Vec<Ciphertext<C::PlainConf>>,
}

impl<C: EncodeConf> EncryptedPolyCode<C>
where
    C::PlainConf: YasheConf,
    <C::PlainConf as PolyConf>::Coeff: From<u128> + From<u64> + From<i64>,
{
    /// Encrypt a PolyCode by encrypting each polynomial.
    pub fn encrypt_code(
        ctx: Yashe<C::PlainConf>,
        code: PolyCode<C>,
        public_key: &PublicKey<C::PlainConf>,
        rng: &mut ThreadRng,
    ) -> Self 
    where 
        C: EncodeConf
    {
        let data = code
            .polys
            .into_iter()
            .map(|p| ctx.encrypt(Message::<C::PlainConf>{ m: p }, &public_key, rng))
            .collect();
        let masks = code
            .masks
            .into_iter()
            .map(|p| ctx.encrypt(Message::<C::PlainConf>{ m: p }, &public_key, rng))
            .collect();
        Self { data, masks }
    }
}

impl<C: EncodeConf> EncryptedPolyQuery<C>
where
    C::PlainConf: YasheConf,
    <C::PlainConf as PolyConf>::Coeff: From<u128> + From<u64> + From<i64>, BigUint: From<<<C as EncodeConf>::PlainConf as PolyConf>::Coeff>
{
    /// Encrypt a PolyQuery by encrypting each polynomial.
    pub fn encrypt_query(
        ctx: Yashe<C::PlainConf>,
        query: PolyQuery<C>,
        public_key: &PublicKey<C::PlainConf>,
        rng: &mut ThreadRng,
    ) -> Self {
        let data = query
            .polys
            .into_iter()
            .map(|p| ctx.encrypt(Message::<C::PlainConf>{ m: p }, &public_key, rng))
            .collect();
        let masks = query
            .masks
            .into_iter()
            .map(|p| ctx.encrypt(Message::<C::PlainConf>{ m: p }, &public_key, rng))
            .collect();
        Self { data, masks }
    }

    /// Returns true if `self` and `code` have enough identical bits to meet the threshold.
    pub fn is_match(
        &self,
        ctx: Yashe<C::PlainConf>,
        private_key: PrivateKey<C::PlainConf>,
        code: &EncryptedPolyCode<C>,
    ) -> Result<bool, MatchError>
    where
        BigUint: From<<C::PlainConf as PolyConf>::Coeff>,
    {
        let match_counts = Self::accumulate_inner_products(ctx, private_key.clone(), &self.data, &code.data)?;
        let mask_counts = Self::accumulate_inner_products(ctx, private_key, &self.masks, &code.masks)?;

        for (d, t) in match_counts.into_iter().zip_eq(mask_counts.into_iter()) {
            // Match if the Hamming distance is less than a percentage threshold:
            // (t - d) / 2t <= x%
            #[allow(clippy::cast_possible_wrap)]
            if (t - d) * (C::EyeConf::MATCH_DENOMINATOR as i64)
                <= 2 * t * (C::EyeConf::MATCH_NUMERATOR as i64)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Similarly to function `accumulate_inner_products`, but return a list containing the products, such that
    /// we can extract inner products later.
    fn accumulate_inner_products(
        ctx: Yashe<C::PlainConf>,
        private_key: PrivateKey<C::PlainConf>,
        a_polys: &[Ciphertext<C::PlainConf>],
        b_polys: &[Ciphertext<C::PlainConf>],
    ) -> Result<Vec<i64>, MatchError> 
    where
        BigUint: From<<C::PlainConf as PolyConf>::Coeff>,
    {
        
        let mut counts = vec![0; C::EyeConf::ROTATION_COMPARISONS];
        // compute T/2 as a big int
        let t_div_2 = BigInt::from(C::PlainConf::T / 2);
        
        for (a, b) in a_polys.iter().zip_eq(b_polys.iter()) {
            // Multiply the polynomials, which will yield inner products.
            let product = ctx.ciphertext_mul(a.clone(), b.clone());
            let decrypted_product = ctx.decrypt_mul(product, &private_key);

            // Extract the inner products from particular coefficients.
            // Left-most rotation:              sδ - (v - u) - 1
            // Right-most rotation (inclusive): sδ - 1
            let block_counts = decrypted_product.m
                .iter()
                .skip(C::ROWS_PER_BLOCK * C::NUM_COLS_AND_PADS - C::EyeConf::ROTATION_COMPARISONS)
                .take(C::EyeConf::ROTATION_COMPARISONS)
                .map(|c| 
                    {
                        let mut coeff_res = C::PlainConf::coeff_as_big_int(*c);
                        let cout = if coeff_res > t_div_2 {
                            coeff_res = C::PlainConf::T - coeff_res;
                            let result = i64::try_from(BigUint::from(C::PlainConf::big_int_as_coeff(coeff_res))).unwrap();
                            Ok(-result)
                        } else {
                            let result = i64::try_from(BigUint::from(C::PlainConf::big_int_as_coeff(coeff_res))).unwrap();
                            Ok(result)
                        };
                        cout
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Accumulate the counts from all blocks, grouped by rotation.
            counts
                .iter_mut()
                .zip(block_counts.into_iter())
                .for_each(|(count, block_count)| {
                    *count += block_count;
                });
        }

        Ok(counts)
    }
}
