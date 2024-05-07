//! Iris matching operations on polynomial-encoded bit vectors.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use itertools::Itertools;
use std::error::Error;

use crate::plaintext::{index_1d, IrisCode, IrisMask};
use crate::primitives::poly::modular_poly::conf::FullRes;
use crate::primitives::poly::{Coeff, Poly, PolyConf};
use crate::{
    IRIS_BIT_LENGTH, IRIS_COLUMNS as NUM_COLS, IRIS_COLUMN_LENGTH as NUM_ROWS,
    IRIS_MATCH_DENOMINATOR, IRIS_MATCH_NUMERATOR, IRIS_ROTATION_COMPARISONS, IRIS_ROTATION_LIMIT,
};
use ark_ff::{One, Zero};
use num_bigint::BigUint;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

// Divide iris codes into blocks that can each fit into a polynomial.
/// The number of rows in each block.
const ROWS_PER_BLOCK: usize = 10;
/// The number of blocks.
const NUM_BLOCKS: usize = NUM_ROWS / ROWS_PER_BLOCK;
// Only full blocks are supported at the moment.
const_assert_eq!(NUM_BLOCKS * ROWS_PER_BLOCK, NUM_ROWS);

// Aliases to match the paper equations.
/// The number of columns in each block.
const K: i32 = NUM_COLS as i32;
/// The right-most rotation.
const V: i32 = IRIS_ROTATION_LIMIT as i32;
/// The left-most rotation.
const U: i32 = -V;
/// The number of columns plus padding for rotations.
const DELTA: usize = (K + V - U) as usize;
/// The number of rows in each block.
const S: usize = ROWS_PER_BLOCK;

/// An Iris code, encoded in polynomials. To be stored in the database.
pub struct PolyCode {
    /// The polynomials, encoding one block of rows each. Storage variant.
    polys: Vec<Poly<FullRes>>,
    /// The mask as plaintext bits.
    mask: IrisMask,
}

/// An Iris code, encoded in polynomials. To be matched against PolyCode.
pub struct PolyQuery {
    /// The polynomials, encoding one block of rows each. Query variant.
    polys: Vec<Poly<FullRes>>,
    /// The mask as plaintext bits.
    mask: IrisMask,
}

impl PolyCode {
    /// Create a new PolyCode from a plaintext IrisCode and IrisMask.
    ///
    /// Reference: Private Iris Matching Protocol, page 40, C_1(a)
    pub fn from_plaintext(value: &IrisCode, mask: &IrisMask) -> Self {
        let polys = (0..NUM_BLOCKS)
            .map(|block_i| {
                let first_row_i = block_i * ROWS_PER_BLOCK;
                Self::from_plaintext_block(value, mask, first_row_i)
            })
            .collect();

        Self { polys, mask: *mask }
    }

    /// Encode one block of rows into one polynomial. Storage variant, equation C_1.
    fn from_plaintext_block(
        value: &IrisCode,
        mask: &IrisMask,
        first_row_i: usize,
    ) -> Poly<FullRes> {
        let mut coeffs = vec![Coeff::zero(); FullRes::MAX_POLY_DEGREE];

        for m in 0..ROWS_PER_BLOCK {
            let row_i = first_row_i + ROWS_PER_BLOCK - 1 - m;

            for i in 0..NUM_COLS {
                let col_i = NUM_COLS - 1 - i;
                let bit_i = index_1d(row_i, col_i);

                if mask[bit_i] {
                    coeffs[DELTA * m + i] = if value[bit_i] {
                        -Coeff::one()
                    } else {
                        Coeff::one()
                    };
                }
            }
        }

        Poly::from_coefficients_vec(coeffs)
    }
}

impl PolyQuery {
    /// Create a new PolyQuery from a plaintext IrisCode and IrisMask.
    ///
    /// Reference: Private Iris Matching Protocol, page 40, C_2(b)
    pub fn from_plaintext(value: &IrisCode, mask: &IrisMask) -> Self {
        let polys = (0..NUM_BLOCKS)
            .map(|block_i| {
                let first_row_i = block_i * ROWS_PER_BLOCK;
                Self::from_plaintext_block(value, mask, first_row_i)
            })
            .collect();

        Self { polys, mask: *mask }
    }

    /// Encode one block of rows into one polynomial. Query variant, equation C_2.
    fn from_plaintext_block(
        value: &IrisCode,
        mask: &IrisMask,
        first_row_i: usize,
    ) -> Poly<FullRes> {
        let mut coeffs = vec![Coeff::zero(); FullRes::MAX_POLY_DEGREE];

        for m in 0..ROWS_PER_BLOCK {
            let row_i = first_row_i + m;

            for i in 0..K + V - U {
                let col_i = (i + U).rem_euclid(K) as usize;
                let bit_i = index_1d(row_i, col_i);

                if mask[bit_i] {
                    coeffs[DELTA * m + i as usize] = if value[bit_i] {
                        -Coeff::one()
                    } else {
                        Coeff::one()
                    };
                }
            }
        }

        Poly::from_coefficients_vec(coeffs)
    }

    /// Returns true if `self` and `code` have enough identical bits to meet the threshold.
    pub fn is_match(&self, code: &PolyCode) -> Result<bool, Box<dyn Error>> {
        let match_counts = self.accumulate_inner_products(code)?;
        let mask_counts = self.count_mask_overlap(code);

        for (d, t) in match_counts.into_iter().zip_eq(mask_counts.into_iter()) {
            // Match if the Hamming distance is less than a percentage threshold:
            // (t - d) / 2t <= 36%
            if (t - d) * (IRIS_MATCH_DENOMINATOR as i64) <= 2 * t * (IRIS_MATCH_NUMERATOR as i64) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Accumulate the inner products of the polynomials for each block of rows.
    /// The result for each rotation is `D = #equal_bits - #different_bits`.
    fn accumulate_inner_products(&self, code: &PolyCode) -> Result<Vec<i64>, Box<dyn Error>> {
        let mut counts = vec![0; IRIS_ROTATION_COMPARISONS];

        for (a, b) in self.polys.iter().zip_eq(code.polys.iter()) {
            // Multiply the polynomials, which will yield inner products.
            let product = a * b;

            // Extract the inner products from particular coefficients.
            let block_counts = product
                .iter()
                .skip(S * DELTA - (V - U) as usize - 1) // From left-most rotation…
                .take((V - U + 1) as usize) // … to right-most rotation.
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

    /// Count the number of bits visible in both the query and the code.
    /// The result for each rotation is `T = #visible_bits`.
    fn count_mask_overlap(&self, code: &PolyCode) -> Vec<i64> {
        let mut query_mask = self.mask;
        query_mask.rotate_left(IRIS_ROTATION_LIMIT * NUM_ROWS); // From left-most rotation…

        (0..IRIS_ROTATION_COMPARISONS)
            .map(|_| {
                let mask = query_mask & code.mask;
                let unmasked = mask.count_ones() as i64;

                query_mask.rotate_right(NUM_ROWS); // … to right-most rotation.
                unmasked
            })
            .collect_vec()
    }
}

/// Convert a prime field element to a signed integer, assuming the range from all equal to all different bits.
fn coeff_to_int(c: Coeff) -> Result<i64, Box<dyn Error>> {
    Ok(if c <= Coeff::from(IRIS_BIT_LENGTH as u64) {
        i64::try_from(BigUint::from(c))?
    } else {
        -i64::try_from(BigUint::from(-c))?
    })
}
