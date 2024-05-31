//! Iris matching operations on polynomial-encoded bit vectors.

use std::error::Error;

use ark_ff::{One, Zero};
use itertools::Itertools;
use num_bigint::BigUint;

use crate::{
    plaintext::{index_1d, IrisCode, IrisMask},
    primitives::poly::{Poly, PolyConf},
    FullRes,
};

pub use conf::EncodeConf;

pub mod conf;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

/// An Iris code, encoded in polynomials. To be stored in the database.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolyCode<C: EncodeConf> {
    /// The polynomials, encoding one block of rows each. Storage variant.
    polys: Vec<Poly<C::PlainConf>>,
    /// The mask polynomials.
    masks: Vec<Poly<C::PlainConf>>,
}

/// An Iris code, encoded in polynomials. To be matched against PolyCode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolyQuery<C: EncodeConf> {
    /// The polynomials, encoding one block of rows each. Query variant.
    polys: Vec<Poly<C::PlainConf>>,
    /// The mask polynomials.
    masks: Vec<Poly<C::PlainConf>>,
}

/// Errors that can happen during matching.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MatchError {
    /// A plaintext coefficient was much larger than expected.
    PlaintextOutOfRange,
}

impl<C: EncodeConf> PolyCode<C> {
    /// Create a new PolyCode from a plaintext IrisCode and IrisMask.
    ///
    /// Reference: Private Iris Matching Protocol, page 40, C_1(a)
    pub fn from_plaintext<const STORE_ELEM_LEN: usize>(
        value: &IrisCode<STORE_ELEM_LEN>,
        mask: &IrisMask<STORE_ELEM_LEN>,
    ) -> Self {
        let polys = (0..NUM_BLOCKS)
            .map(|block_i| {
                let first_row_i = block_i * ROWS_PER_BLOCK;
                Self::from_plaintext_block(value, mask, first_row_i)
            })
            .collect_vec();

        let masks = polys.iter().map(poly_bits_to_masks).collect();

        Self { polys, masks }
    }

    /// Encode one block of rows into one polynomial. Storage variant, equation C_1.
    fn from_plaintext_block<const STORE_ELEM_LEN: usize>(
        value: &IrisCode<STORE_ELEM_LEN>,
        mask: &IrisMask<STORE_ELEM_LEN>,
        first_row_i: usize,
    ) -> Poly<C::PlainConf> {
        let mut coeffs = Poly::non_canonical_zeroes(PlainConf::MAX_POLY_DEGREE);

        for m in 0..ROWS_PER_BLOCK {
            let row_i = first_row_i + ROWS_PER_BLOCK - 1 - m;

            // Set the coefficients of C₁ = ∑ aⱼ * xⁱ
            // i ∈ [0, k - 1]
            // j = k - 1 - i
            for i in 0..NUM_COLS {
                let col_i = NUM_COLS - 1 - i;
                let bit_i = index_1d(row_i, col_i);

                if mask[bit_i] {
                    coeffs[NUM_COLS_AND_PADS * m + i] = if value[bit_i] {
                        -PlainCoeff::one()
                    } else {
                        PlainCoeff::one()
                    };
                }
            }
        }

        coeffs.truncate_to_canonical_form();
        coeffs
    }
}

impl<C: EncodeConf> PolyQuery<C> {
    /// Create a new PolyQuery from a plaintext IrisCode and IrisMask.
    ///
    /// Reference: Private Iris Matching Protocol, page 40, C_2(b)
    pub fn from_plaintext<const STORE_ELEM_LEN: usize>(
        value: &IrisCode<STORE_ELEM_LEN>,
        mask: &IrisMask<STORE_ELEM_LEN>,
    ) -> Self {
        // This code is textually the same as PolyCode::from_plaintext, but the
        // from_plaintext_block() method is different.
        let polys = (0..NUM_BLOCKS)
            .map(|block_i| {
                let first_row_i = block_i * ROWS_PER_BLOCK;
                Self::from_plaintext_block(value, mask, first_row_i)
            })
            .collect_vec();

        let masks = polys.iter().map(poly_bits_to_masks).collect();

        Self { polys, masks }
    }

    /// Encode one block of rows into one polynomial. Query variant, equation C_2.
    fn from_plaintext_block<const STORE_ELEM_LEN: usize>(
        value: &IrisCode<STORE_ELEM_LEN>,
        mask: &IrisMask<STORE_ELEM_LEN>,
        first_row_i: usize,
    ) -> Poly<C::PlainConf> {
        let mut coeffs = Poly::non_canonical_zeroes(PlainConf::MAX_POLY_DEGREE);

        for m in 0..ROWS_PER_BLOCK {
            let row_i = first_row_i + m;

            // Set the coefficients of C₂ = ∑ aⱼ * xⁱ
            // i = j - u
            // j ∈ [u, k - 1 + v]
            // aⱼ is indexed with j mod k.
            for i in 0..NUM_COLS_AND_PADS {
                #[allow(clippy::cast_possible_wrap)]
                let col_i = {
                    let j = i as isize - (IRIS_ROTATION_LIMIT as isize);
                    j.rem_euclid(NUM_COLS as isize) as usize
                };
                let bit_i = index_1d(row_i, col_i);

                if mask[bit_i] {
                    coeffs[NUM_COLS_AND_PADS * m + i] = if value[bit_i] {
                        -PlainCoeff::one()
                    } else {
                        PlainCoeff::one()
                    };
                }
            }
        }

        coeffs.truncate_to_canonical_form();
        coeffs
    }

    /// Returns true if `self` and `code` have enough identical bits to meet the threshold.
    pub fn is_match(&self, code: &PolyCode<C>) -> Result<bool, Box<dyn Error>> {
        let match_counts = accumulate_inner_products(&self.polys, &code.polys)?;
        let mask_counts = accumulate_inner_products(&self.masks, &code.masks)?;

        for (d, t) in match_counts.into_iter().zip_eq(mask_counts.into_iter()) {
            // Match if the Hamming distance is less than a percentage threshold:
            // (t - d) / 2t <= x%
            #[allow(clippy::cast_possible_wrap)]
            if (t - d) * (IRIS_MATCH_DENOMINATOR as i64) <= 2 * t * (IRIS_MATCH_NUMERATOR as i64) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Accumulate the inner products of the polynomials for each block of rows.
    /// The result for each rotation is `D = #equal_bits - #different_bits`.
    fn accumulate_inner_products(
        a_polys: &[Poly<FullRes>],
        b_polys: &[Poly<FullRes>],
    ) -> Result<Vec<i64>, Box<dyn Error>> {
        let mut counts = vec![0; NUM_ROTATIONS];

        for (a, b) in a_polys.iter().zip_eq(b_polys.iter()) {
            // Multiply the polynomials, which will yield inner products.
            let product = a * b;

            // Extract the inner products from particular coefficients.
            // Left-most rotation:              sδ - (v - u) - 1
            // Right-most rotation (inclusive): sδ - 1
            let block_counts = product
                .iter()
                .skip(ROWS_PER_BLOCK * NUM_COLS_AND_PADS - NUM_ROTATIONS)
                .take(NUM_ROTATIONS)
                .map(|c| coeff_to_int(*c))
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

    /// Create a mask polynomial from a polynomial of encoded bits.
    fn poly_bits_to_masks(bits: &Poly<PlainConf>) -> Poly<PlainConf> {
        let mut masks = Poly::non_canonical_zeroes(PlainConf::MAX_POLY_DEGREE);
        for i in 0..PlainConf::MAX_POLY_DEGREE {
            masks[i] = if bits[i].is_zero() {
                PlainCoeff::zero()
            } else {
                PlainCoeff::one()
            };
        }
        masks.truncate_to_canonical_form();
        masks
    }
}


