//! Iris matching operations on polynomial-encoded bit vectors.

use itertools::Itertools;

use crate::plaintext::{IrisCode, IrisMask};
use crate::primitives::poly::{Coeff, Poly, FULL_RES_POLY_DEGREE};
use crate::{
    IRIS_BIT_LENGTH, IRIS_COLUMNS as NUM_COLS, IRIS_COLUMN_LENGTH as NUM_ROWS,
    IRIS_MATCH_DENOMINATOR, IRIS_MATCH_NUMERATOR, IRIS_ROTATION_COMPARISONS, IRIS_ROTATION_LIMIT,
};
use ark_ff::{BigInt, One, PrimeField, Zero};
use num_bigint::BigUint;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

const ROWS_PER_BLOCK: usize = 10;
const NUM_BLOCKS: usize = NUM_ROWS / ROWS_PER_BLOCK;

/// An Iris code, encoded in polynomials. To be stored in the database.
pub struct PolyCode {
    polys: Vec<Poly<FULL_RES_POLY_DEGREE>>,
    mask: IrisMask,
}

/// An Iris code, encoded in polynomials. To be matched against PolyCode.
pub struct PolyQuery {
    polys: Vec<Poly<FULL_RES_POLY_DEGREE>>,
    mask: IrisMask,
}

impl PolyCode {
    /// Create a new PolyCode from a plaintext IrisCode and IrisMask.
    ///
    /// Reference: Private Iris Matching Protocol, page 40, C₁(a)
    pub fn from_plaintext(value: &IrisCode, mask: &IrisMask) -> Self {
        let mut polys = vec![];
        for block_i in 0..NUM_BLOCKS {
            let poly = Self::from_plaintext_block(value, mask, block_i);
            polys.push(poly);
        }

        Self { polys, mask: *mask }
    }

    fn from_plaintext_block(
        value: &IrisCode,
        mask: &IrisMask,
        block_i: usize,
    ) -> Poly<FULL_RES_POLY_DEGREE> {
        let k = NUM_COLS as i32;
        let v: i32 = IRIS_ROTATION_LIMIT as i32;
        let u = -v;
        let delta = (k + v - u) as usize;

        let mut coeffs = vec![Coeff::zero(); FULL_RES_POLY_DEGREE];
        for m in 0..ROWS_PER_BLOCK {
            let row_i = block_i * ROWS_PER_BLOCK + ROWS_PER_BLOCK - 1 - m;

            for i in 0..NUM_COLS {
                let col_i = NUM_COLS - 1 - i;
                // To column-major vector.
                let bit_i = col_i * NUM_ROWS + row_i;

                if mask[bit_i] {
                    coeffs[delta * m + i] = if value[bit_i] {
                        -Coeff::one()
                    } else {
                        Coeff::one()
                    };
                }
            }
        }
        // TODO: get rows from an IrisCode method to abstract the column-major encoding.

        Poly::from_coefficients_vec(coeffs)
    }
}

impl PolyQuery {
    /// Create a new PolyQuery from a plaintext IrisCode and IrisMask.
    ///
    /// Reference: Private Iris Matching Protocol, page 40, C₂(b)
    pub fn from_plaintext(value: &IrisCode, mask: &IrisMask) -> Self {
        let mut polys = vec![];
        for block_i in 0..NUM_BLOCKS {
            let poly = Self::from_plaintext_block(value, mask, block_i);
            polys.push(poly);
        }

        Self { polys, mask: *mask }
    }

    fn from_plaintext_block(
        value: &IrisCode,
        mask: &IrisMask,
        block_i: usize,
    ) -> Poly<FULL_RES_POLY_DEGREE> {
        let k = NUM_COLS as i32;
        let v: i32 = IRIS_ROTATION_LIMIT as i32;
        let u = -v;
        let delta = (k + v - u) as usize;

        let mut coeffs = vec![Coeff::zero(); FULL_RES_POLY_DEGREE];
        for m in 0..ROWS_PER_BLOCK {
            let row_i = block_i * ROWS_PER_BLOCK + m;

            for i in 0..k + v - u {
                let col_i = (i + u).rem_euclid(k) as usize;
                // To column-major vector.
                let bit_i = col_i * NUM_ROWS + row_i;

                if mask[bit_i] {
                    coeffs[delta * m + i as usize] = if value[bit_i] {
                        -Coeff::one()
                    } else {
                        Coeff::one()
                    };
                }
            }
        }
        // TODO: get rows from an IrisCode method to abstract the column-major encoding.

        Poly::from_coefficients_vec(coeffs)
    }

    /// Returns true if `self` and `code` have enough identical bits to meet the threshold.
    pub fn is_match(&self, code: &PolyCode) -> bool {
        let k = NUM_COLS as i32;
        let v: i32 = IRIS_ROTATION_LIMIT as i32;
        let u = -v;
        let s = ROWS_PER_BLOCK;
        let delta = (k + v - u) as usize;

        // TODO: all polys.
        let product = &self.polys[0] * &code.polys[0];
        let differences = product
            .iter()
            .skip(s * delta - (v - u) as usize - 1) // From left-most rotation…
            .take((v - u + 1) as usize) // … to right-most rotation.
            .map(|c| coeff_to_int(*c))
            .collect::<Vec<i64>>();

        // TODO: simplify, compute the mask overlap sizes separately.
        let mut query_mask = self.mask;
        query_mask.rotate_left(IRIS_ROTATION_LIMIT * NUM_ROWS); // From left-most rotation…
        let mut mask_counts: Vec<i64> = vec![];

        for _rotation in 0..IRIS_ROTATION_COMPARISONS {
            let mask = query_mask & code.mask;
            let unmasked = mask.count_ones() as i64;

            mask_counts.push(unmasked);
            query_mask.rotate_right(NUM_ROWS); // … to right-most rotation.
        }

        for (d, t) in differences.into_iter().zip(mask_counts.into_iter()) {
            //let t = BigUint::from(t);
            let denom = BigUint::from(IRIS_MATCH_DENOMINATOR as u64);
            let numer = BigUint::from(IRIS_MATCH_NUMERATOR as u64);
            let two = BigUint::from(2u64);

            // Match if the Hamming distance is less than a percentage threshold:
            // (t - d) / 2t <= 36%
            //if (t - d) * denom <= two * t * numer {
            if (t - d) * (IRIS_MATCH_DENOMINATOR as i64) <= 2 * t * (IRIS_MATCH_NUMERATOR as i64) {
                return true;
            }
        }

        false
    }
}

// TODO: validation and error handling.
fn coeff_to_int(c: Coeff) -> i64 {
    if c >= Coeff::zero() {
        let bi = BigUint::from(c);
        let bi = i64::try_from(bi).unwrap();
        bi
    } else {
        let bi = BigUint::from(-c);
        let bi = i64::try_from(bi).unwrap();
        -bi
    }
}
