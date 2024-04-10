//! The implementation of a modular polynomial, [`Poly`].
//!
//! This module contains fundamental operations which ensure that the polynomial is always in its canonical form.
//! They should be called after every operation that can create non-canonical polynomials, which can happen when:
//! - the leading coefficient is set to zero, including when the polynomial is split or truncated, or
//! - the degree of the polynomial is increased, for example, during multiplication.

// Optional TODOs:
// - re-implement IndexMut manually, to enforce the canonical form (highest coefficient is non-zero) and modular arithmetic
//   (this can be done by returning a new type with `DerefMut<Target = Coeff>``, but it could have performance impacts)
// Trivial:
// - implement Sum manually

use std::ops::{Index, IndexMut, Mul};

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

    /// Reduce this polynomial so it is less than [`POLY_MODULUS`](static@POLY_MODULUS).
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

    /// Multiplies then reduces by [`POLY_MODULUS`](static@POLY_MODULUS).
    fn mul(self, rhs: Self) -> Self {
        let mut res = Self(&self.0 * &rhs.0);
        res.reduce_mod_poly();
        res
    }
}

impl Index<usize> for Poly {
    type Output = Coeff;

    /// A trivial index forwarding implementation that panics.
    ///
    /// Panics indicate redundant code which should have stopped at the highest non-zero coefficient.
    /// Using `self.coeffs.iter()` is one way to ensure the code only accesses real indexes.
    fn index(&self, index: usize) -> &Self::Output {
        self.coeffs.get(index).expect(
            "accessed redundant zero coefficient: \
            improve performance by stopping at the highest non-zero coefficient",
        )
    }
}

impl IndexMut<usize> for Poly {
    /// An auto-expanding index implementation that can set any coefficient without panicking.
    /// Use this implementation via `poly[index]`.
    ///
    /// # Correct Usage
    ///
    /// After setting the index, the caller must reduce the polynomial and restore canonical form,
    /// by calling `Poly::reduce_mod_poly()`.
    ///
    /// Using `poly.coeffs[index]` can panic, because it skip this implementation, using `<Vec as IndexMut>` instead.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // Make sure there is a coefficient entry for `index`, but don't truncate if the index already exists.
        if index + 1 > self.coeffs.len() {
            self.coeffs.resize(index + 1, Coeff::zero());
        }

        self.coeffs.get_mut(index).expect("just resized")
    }
}

/// The fastest available modular polynomial operation.
pub use mod_poly_manual_mut as mod_poly;

/// Reduces `dividend` to `dividend % [POLY_MODULUS]`.
///
/// This is the most efficient manual implementation.
pub fn mod_poly_manual_mut(dividend: &mut Poly) {
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

    // The DenseOrSparsePolynomial implementation ensures canonical form.
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
