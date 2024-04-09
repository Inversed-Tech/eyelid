//! The implementation of a modular polynomial, [`Poly`].
//!
//! This module calls the base operations from [`super`] to ensure that the polynomial is always in its canonical form.

use std::ops::Mul;

use ark_ff::{One, Zero};
use ark_poly::polynomial::{
    univariate::{DenseOrSparsePolynomial, DensePolynomial},
    Polynomial,
};
use derive_more::{
    Add, AsRef, Deref, DerefMut, Div, DivAssign, From, Into, Mul, MulAssign, Neg, Rem, RemAssign,
};
use lazy_static::lazy_static;

use crate::primitives::poly::{Coeff, MAX_POLY_DEGREE};

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
    Deref,
    // TODO: manually implement a final reduce step
    DerefMut,
    // TODO: manually implement a final reduce step
    From,
    Into,
    Neg,
    Add,
    // TODO: manually implement a final reduce step
    Mul,
    // TODO: manually implement a final reduce step
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
        let mut new = Self(DensePolynomial { coeffs });
        new.truncate_to_canonical_form();
        new
    }

    /// Converts the `coeffs` slice into a dense polynomial.
    pub fn from_coefficients_slice(coeffs: &[Coeff]) -> Self {
        Self::from_coefficients_vec(coeffs.to_vec())
    }

    // Basic Internal Operations

    /// Reduce this polynomial so it is less than [`POLY_MODULUS`].
    /// This also ensures its degree is less than [`MAX_POLY_DEGREE`].
    ///
    /// This operation should be performed after every [`Poly`] method that increases the degree of the polynomial.
    /// [`DensePolynomial`] methods *do not* do this reduction.
    pub fn reduce_mod_poly(&mut self) {
        mod_poly(self);
    }

    /// Truncate this polynomial so it is in the valid canonical form expected by [`DensePolynomial`] methods.
    ///
    /// This operation must be performed after every [`Poly`] method that changes the degree or coefficients of the polynomial.
    /// (`DensePolynomial` methods already do this.)
    pub fn truncate_to_canonical_form(&mut self) {
        while self.coeffs.last() == Some(&Coeff::zero()) {
            self.coeffs.pop();
        }
    }
}

impl Mul for Poly {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut res = Self(&self.0 * &rhs.0);
        res.reduce_mod_poly();
        res
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
pub use mod_poly_manual_mut as mod_poly;

/// Reduces `dividend` to `dividend % [POLY_MODULUS]`.
///
/// This is the most efficient manual implementation.
pub fn mod_poly_manual_mut(dividend: &mut Poly) {
    let mut i = MAX_POLY_DEGREE;
    while i < dividend.coeffs.len() {
        // In the cyclotomic ring we have that XˆN = -1,
        // therefore all elements from N to 2N-1 are negated.

        let q = i / MAX_POLY_DEGREE;
        let r = i % MAX_POLY_DEGREE;
        if q % 2 == 1 {
            dividend[r] = dividend[r] - dividend[i];
        } else {
            dividend[r] = dividend[r] + dividend[i];
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
pub fn mod_poly_manual_ref(dividend: &Poly) -> Poly {
    let mut dividend = dividend.clone();
    mod_poly_manual_mut(&mut dividend);
    dividend
}

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This uses an [`ark-poly`] library implementation, which always creates a new polynomial.
pub fn mod_poly_ark_ref(dividend: &Poly) -> Poly {
    let dividend: DenseOrSparsePolynomial<'_, _> = dividend.into();

    let (_quotient, remainder) = dividend
        .divide_with_q_and_r(&*POLY_MODULUS)
        .expect("POLY_MODULUS is not zero");

    remainder.into()
}

/// Reduces `dividend` to `dividend % [POLY_MODULUS]`.
///
/// This uses an [`ark-poly`] library implementation, and entirely replaces the inner polynomial representation.
pub fn mod_poly_ark_mut(dividend: &mut Poly) {
    let remainder = mod_poly_ark_ref(dividend);
    *dividend = remainder;
}
