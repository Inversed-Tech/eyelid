//! Iris matching operations on polynomial-encoded bit vectors.

use itertools::Itertools;
use std::error::Error;

use crate::plaintext::{index_1d, IrisCode, IrisMask};
use crate::primitives::poly::modular_poly::conf::FullRes;
use crate::primitives::poly::{Poly, PolyConf};
use crate::{
    IRIS_BIT_LENGTH,
    IRIS_COLUMNS as NUM_COLS, // The number of columns of the code: `k`
    IRIS_COLUMN_LENGTH as NUM_ROWS,
    IRIS_MATCH_DENOMINATOR,
    IRIS_MATCH_NUMERATOR,
    IRIS_ROTATION_COMPARISONS as NUM_ROTATIONS, // The number of rotations: `v - u + 1`
    IRIS_ROTATION_LIMIT,                        // The rotation limits: `v` and `u = -v`
};
use ark_ff::{One, Zero};
use num_bigint::BigUint;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

/// The configuration of plaintext polynomials.
type PlainConf = FullRes;

/// The type of the coefficients of plaintext polynomials.
type PlainCoeff = <PlainConf as PolyConf>::Coeff;

// Divide iris codes into blocks that can each fit into a polynomial.
/// The number of rows in each block: `s`
const ROWS_PER_BLOCK: usize = 10;
/// The number of blocks necessary to hold all rows of the code.
const NUM_BLOCKS: usize = NUM_ROWS / ROWS_PER_BLOCK;
// Only full blocks are supported at the moment.
const_assert_eq!(NUM_BLOCKS * ROWS_PER_BLOCK, NUM_ROWS);

/// The number of columns plus padding for rotations: δ = k + v - u
const NUM_COLS_AND_PADS: usize = NUM_COLS + 2 * IRIS_ROTATION_LIMIT;

/// An Iris code, encoded in polynomials. To be stored in the database.
pub struct PolyCode {
    /// The polynomials, encoding one block of rows each. Storage variant.
    polys: Vec<Poly<PlainConf>>,
    /// The mask polynomials.
    masks: Vec<Poly<PlainConf>>,
}

/// An Iris code, encoded in polynomials. To be matched against PolyCode.
pub struct PolyQuery {
    /// The polynomials, encoding one block of rows each. Query variant.
    polys: Vec<Poly<PlainConf>>,
    /// The mask polynomials.
    masks: Vec<Poly<PlainConf>>,
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
            .collect_vec();

        let masks = polys.iter().map(poly_bits_to_masks).collect();

        Self { polys, masks }
    }

    /// Encode one block of rows into one polynomial. Storage variant, equation C_1.
    fn from_plaintext_block(
        value: &IrisCode,
        mask: &IrisMask,
        first_row_i: usize,
    ) -> Poly<PlainConf> {
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
            .collect_vec();

        let masks = polys.iter().map(poly_bits_to_masks).collect();

        Self { polys, masks }
    }

    /// Encode one block of rows into one polynomial. Query variant, equation C_2.
    fn from_plaintext_block(
        value: &IrisCode,
        mask: &IrisMask,
        first_row_i: usize,
    ) -> Poly<PlainConf> {
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
    pub fn is_match(&self, code: &PolyCode) -> Result<bool, Box<dyn Error>> {
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

/// Convert a prime field element to a signed integer, assuming the range from all equal to all different bits.
fn coeff_to_int(c: PlainCoeff) -> Result<i64, Box<dyn Error>> {
    Ok(if c <= PlainCoeff::from(IRIS_BIT_LENGTH as u64) {
        i64::try_from(BigUint::from(c))?
    } else {
        -i64::try_from(BigUint::from(-c))?
    })
}
