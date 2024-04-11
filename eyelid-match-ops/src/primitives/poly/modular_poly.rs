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
use ark_poly::polynomial::univariate::{
    DenseOrSparsePolynomial, DensePolynomial, SparsePolynomial,
};
use derive_more::{Add, AsRef, Deref, DerefMut, Div, Into, Neg, Rem};

use crate::primitives::poly::{mod_poly, mul_poly, Coeff};

// Doc links
#[allow(unused_imports)]
use crate::primitives::poly::{modular_poly::modulus::POLY_MODULUS, MAX_POLY_DEGREE};

pub(super) mod modulus;

mod trivial;

/// A modular polynomial with coefficients in [`Coeff`], and maximum degree [`MAX_POLY_DEGREE`].
/// The un-reduced polynomial modulus is [`POLY_MODULUS`](static@POLY_MODULUS).
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
    Into,
    Neg,
    Add,
    // We don't implement DivAssign and RemAssign, because they have hidden clones.
    Div,
    Rem,
)]
pub struct Poly(DensePolynomial<Coeff>);

impl Poly {
    // Shadow DenseUVPolynomial methods, so we don't have to implement Polynomial and all its supertraits.

    /// Converts the `coeffs` vector into a dense polynomial.
    pub fn from_coefficients_vec(coeffs: Vec<Coeff>) -> Self {
        let mut poly = Self(DensePolynomial { coeffs });
        poly.truncate_to_canonical_form();
        poly
    }

    /// Converts the `coeffs` slice into a dense polynomial.
    pub fn from_coefficients_slice(coeffs: &[Coeff]) -> Self {
        Self::from_coefficients_vec(coeffs.to_vec())
    }

    // Efficient Re-Implementations

    /// Returns `X^n` as a polynomial in reduced form.
    pub fn xn(n: usize) -> Self {
        let mut poly = Self::zero();
        poly[n] = Coeff::one();

        poly.reduce_mod_poly();

        poly
    }

    /// Multiplies `self` by `X^n`, then reduces if needed.
    pub fn mul_xn(&mut self, n: usize) {
        // Puts `n` zeroes as the highest coefficients of the polynomial.
        let new_len = self.coeffs.len() + n;
        self.coeffs.resize(new_len, Coeff::zero());

        // Moves those `n` zeroes to the lowest coefficients of the polynomial, and shifts the rest up.
        self.coeffs.rotate_right(n);

        self.reduce_mod_poly();
    }

    /// Divides `self` by `X^n`, and returns `(quotient, remainder)`.
    pub fn div_xn(mut self, n: usize) -> (Self, Self) {
        // Make `self` the remainder by splitting off the quotient.
        let quotient = self.coeffs.split_off(n);

        (Self::from_coefficients_vec(quotient), self)
    }

    // Basic Internal Operations

    /// Multiplies two polynomials, and returns the result in reduced form.
    ///
    /// This operation can be called using the `*` operator, this method is only needed to disambiguate.
    pub fn mul_reduce(&self, rhs: &Self) -> Poly {
        mul_poly(self, rhs)
    }

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

impl From<DensePolynomial<Coeff>> for Poly {
    fn from(poly: DensePolynomial<Coeff>) -> Self {
        let mut poly = Self(poly);
        poly.reduce_mod_poly();
        poly
    }
}

impl From<&DensePolynomial<Coeff>> for Poly {
    fn from(poly: &DensePolynomial<Coeff>) -> Self {
        let mut poly = Self(poly.clone());
        poly.reduce_mod_poly();
        poly
    }
}

// These are less likely to be called, so it's ok to have sub-optimal performance.
impl From<SparsePolynomial<Coeff>> for Poly {
    fn from(poly: SparsePolynomial<Coeff>) -> Self {
        DensePolynomial::from(poly).into()
    }
}

impl From<&SparsePolynomial<Coeff>> for Poly {
    fn from(poly: &SparsePolynomial<Coeff>) -> Self {
        DensePolynomial::from(poly.clone()).into()
    }
}

impl<'a> From<DenseOrSparsePolynomial<'a, Coeff>> for Poly {
    fn from(poly: DenseOrSparsePolynomial<'a, Coeff>) -> Self {
        DensePolynomial::from(poly).into()
    }
}

impl<'a> From<&DenseOrSparsePolynomial<'a, Coeff>> for Poly {
    fn from(poly: &DenseOrSparsePolynomial<'a, Coeff>) -> Self {
        DensePolynomial::from(poly.clone()).into()
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
            "accessed virtual leading zero coefficient: \
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

// We don't implement operators for SparsePolynomial or DenseOrSparsePolynomial, they are rare and can use .into() to convert first.
impl Mul for Poly {
    type Output = Self;

    /// Multiplies then reduces by [`POLY_MODULUS`](static@POLY_MODULUS).
    fn mul(self, rhs: Self) -> Self {
        mul_poly(&self, &rhs)
    }
}

impl Mul<&Poly> for Poly {
    type Output = Self;

    /// Multiplies then reduces by [`POLY_MODULUS`](static@POLY_MODULUS).
    fn mul(self, rhs: &Self) -> Self {
        mul_poly(&self, rhs)
    }
}

impl Mul<DensePolynomial<Coeff>> for Poly {
    type Output = Self;

    /// Multiplies then reduces by [`POLY_MODULUS`](static@POLY_MODULUS).
    fn mul(self, rhs: DensePolynomial<Coeff>) -> Self {
        mul_poly(&self, &Self(rhs))
    }
}

// TODO: if we need this method, remove the clone() using unsafe code
#[cfg(inefficient)]
impl Mul<&DensePolynomial<Coeff>> for Poly {
    type Output = Self;

    /// Multiplies then reduces by [`POLY_MODULUS`](static@POLY_MODULUS).
    fn mul(self, rhs: &DensePolynomial<Coeff>) -> Self {
        mul_poly(&self, &Self(rhs.clone()))
    }
}
