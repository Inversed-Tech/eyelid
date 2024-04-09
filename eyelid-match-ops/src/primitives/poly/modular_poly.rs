//! The implementation of a modular polynomial, [`Poly`].
//!
//! This module calls the base operations from [`super`] to ensure that the polynomial is always in its canonical form.

use ark_ff::{One, Zero};
use ark_poly::polynomial::{
    univariate::{DenseOrSparsePolynomial, DensePolynomial},
    Polynomial,
};
use derive_more::{
    Add, AsRef, Constructor, Deref, DerefMut, Div, DivAssign, From, Index, IndexMut, Into, Mul,
    MulAssign, Neg, Rem, RemAssign,
};
use lazy_static::lazy_static;

use super::Coeff;

// For doc links
#[allow(unused_imports)]
use super::MAX_POLY_DEGREE;

mod trivial;

lazy_static! {
    /// The polynomial modulus used for the polynomial field, `X^[MAX_POLY_DEGREE] + 1`.
    /// This means that `X^[MAX_POLY_DEGREE] = -1`.
    pub static ref POLY_MODULUS: DenseOrSparsePolynomial<'static, Coeff> = {
        let mut poly = zero_poly(MAX_POLY_DEGREE);

        poly[MAX_POLY_DEGREE] = Coeff::one();
        poly[0] = Coeff::one();

        assert_eq!(poly.degree(), MAX_POLY_DEGREE);

        poly.into()
    };
}

/// A modular polynomial with coefficients in [`Coeff`], and maximum degree [`MAX_POLY_DEGREE`].
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    AsRef,
    Constructor,
    Deref,
    DerefMut,
    From,
    Into,
    Index,
    IndexMut,
    Neg,
    Add,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Rem,
    RemAssign,
)]
pub struct Poly(DensePolynomial<Coeff>);

// TODO:
// - enforce the constant degree MAX_POLY_DEGREE
// - re-implement Index and IndexMut manually, to enforce the canonical form (highest coefficient is non-zero) and modular arithmetic
// - re-implement Mul and MulAssign manually, to enforce modular arithmetic by POLY_MODULUS (Add, Sub, Div, Rem, and Neg can't increase the degree)
impl Poly {
    // Shadow DenseUVPolynomial methods, so we don't have to implement Polynomial and all its supertraits.

    /// Converts the `coeffs` vector into a dense polynomial.
    pub fn from_coefficients_vec(coeffs: Vec<Coeff>) -> Self {
        Self(DensePolynomial { coeffs })
    }

    /// Converts the `coeffs` slice into a dense polynomial.
    pub fn from_coefficients_slice(coeffs: &[Coeff]) -> Self {
        Self::from_coefficients_vec(coeffs.to_vec())
    }
}

/// Returns the zero polynomial with `degree`.
///
/// This is not the canonical form, but it's useful for creating other polynomials.
/// (Non-canonical polynomials will panic when `degree()` is called on them.)
pub fn zero_poly(degree: usize) -> Poly {
    assert!(degree <= MAX_POLY_DEGREE);

    let mut poly = Poly::zero();
    poly.coeffs = vec![Coeff::zero(); degree + 1];
    poly
}

/// The fastest available modular polynomial operation.
pub use mod_poly_manual as mod_poly;

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This is a manual implementation.
pub fn mod_poly_manual(dividend: &Poly) -> Poly {
    let mut res = dividend.clone();

    let mut i = MAX_POLY_DEGREE;
    while i < res.coeffs.len() {
        // In the cyclotomic ring we have that XˆN = -1,
        // therefore all elements from N to 2N-1 are negated.

        let q = i / MAX_POLY_DEGREE;
        let r = i % MAX_POLY_DEGREE;
        if q % 2 == 1 {
            res[r] = res[r] - res[i];
        } else {
            res[r] = res[r] + res[i];
        }
        i += 1;
    }

    // These elements have already been negated and summed above.
    res.coeffs.truncate(MAX_POLY_DEGREE);

    // Leading elements might be zero, so make sure the polynomial is in the canonical form.
    while res.coeffs.last() == Some(&Coeff::zero()) {
        res.coeffs.pop();
    }

    res
}

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This uses an [`ark-poly`] library implementation.
pub fn mod_poly_ark(dividend: &Poly) -> Poly {
    let dividend: DenseOrSparsePolynomial<'_, _> = dividend.into();

    let (_quotient, remainder) = dividend
        .divide_with_q_and_r(&*POLY_MODULUS)
        .expect("POLY_MODULUS is not zero");

    remainder.into()
}
