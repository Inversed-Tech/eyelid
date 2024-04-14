//! Reduction by the polynomial modulus `X^[MAX_POLY_DEGREE] + 1`.

use ark_ff::{One, Zero};
use ark_poly::polynomial::{univariate::DenseOrSparsePolynomial, Polynomial};
use lazy_static::lazy_static;

use crate::primitives::poly::{Coeff, Poly};

/// The maximum exponent in the polynomial.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
/// N = 2048
#[cfg(not(tiny_poly))]
pub const FULL_RES_POLY_DEGREE: usize = 2048;

/// The maximum exponent in the test-only polynomial.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.
/// N = 5
#[cfg(tiny_poly)]
pub const FULL_RES_POLY_DEGREE: usize = 5;

/// The fastest available modular polynomial operation.
pub use mod_poly_manual_mut as mod_poly;

/// Reduces `dividend` to `dividend % [POLY_MODULUS]`.
///
/// This is the most efficient manual implementation.
pub fn mod_poly_manual_mut<const MAX_POLY_DEGREE: usize>(dividend: &mut Poly<MAX_POLY_DEGREE>) {
    let mut i = MAX_POLY_DEGREE;
    while i < dividend.coeffs.len() {
        let q = i / MAX_POLY_DEGREE;
        let r = i % MAX_POLY_DEGREE;

        // In the cyclotomic ring we have that XˆN = -1,
        // therefore all elements from N to 2N-1 are negated.
        //
        // For performance reasons, we use <Vec as IndexMut>,
        // because the loop condition limits `i` to valid indexes.
        if q % 2 == 1 {
            dividend.coeffs[r] = dividend.coeffs[r] - dividend.coeffs[i];
        } else {
            dividend.coeffs[r] = dividend.coeffs[r] + dividend.coeffs[i];
        }
        i += 1;
    }

    // The coefficients of MAX_POLY_DEGREE and higher have already been summed above.
    dividend.coeffs.truncate(MAX_POLY_DEGREE);

    // The coefficients could sum to zero, so make sure the polynomial is in the canonical form.
    dividend.truncate_to_canonical_form();
}

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This clones then uses the manual implementation.
#[cfg(inefficient)]
pub fn mod_poly_manual_ref<const MAX_POLY_DEGREE: usize>(
    dividend: &Poly<MAX_POLY_DEGREE>,
) -> Poly<MAX_POLY_DEGREE> {
    let mut dividend = dividend.clone();
    mod_poly_manual_mut(&mut dividend);
    dividend
}

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This uses an [`ark-poly`] library implementation, which always creates a new polynomial.
pub fn mod_poly_ark_ref<const MAX_POLY_DEGREE: usize>(
    dividend: &Poly<MAX_POLY_DEGREE>,
) -> Poly<MAX_POLY_DEGREE> {
    lazy_static! {
        /// The polynomial modulus used for the polynomial field, `X^[MAX_POLY_DEGREE] + 1`.
        /// This means that `X^[MAX_POLY_DEGREE] = -1`.
        ///
        /// This is the canonical but un-reduced form of the modulus, because the reduced form is the zero polynomial.
        pub static ref POLY_MODULUS: DenseOrSparsePolynomial<'static, Coeff> = {
            let mut poly = Poly::zero();

            // Since the leading coefficient is non-zero, this is in canonical form.
            // Resize to the maximum size first, to avoid repeated reallocations.
            poly[MAX_POLY_DEGREE] = Coeff::one();
            poly[0] = Coeff::one();

            // Check canonicity and degree.
            assert_eq!(poly.degree(), MAX_POLY_DEGREE);

            poly.into()
        };
    }

    let dividend: DenseOrSparsePolynomial<'_, _> = dividend.into();

    // The DenseOrSparsePolynomial implementation ensures canonical form.
    let (_quotient, remainder) = dividend
        .divide_with_q_and_r(&*POLY_MODULUS)
        .expect("POLY_MODULUS is not zero");

    remainder.into()
}

/// Reduces `dividend` to `dividend % [POLY_MODULUS]`.
///
/// This uses an [`ark-poly`] library implementation, and entirely replaces the inner polynomial representation.
#[cfg(inefficient)]
pub fn mod_poly_ark_mut<const MAX_POLY_DEGREE: usize>(dividend: &mut Poly<MAX_POLY_DEGREE>) {
    let remainder = mod_poly_ark_ref(dividend);
    *dividend = remainder;
}
