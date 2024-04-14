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
///
/// In its canonical form, a polynomial is a list of coefficients from the constant term `X^0`
/// to the degree `X^n`, where the highest coefficient is non-zero. Leading zero coefficients are
/// not stored.
///
/// There is one more coefficient than the degree, because of the constant term. If the polynomial
/// is the zero polynomial, the degree is `0`, and there are no coefficients.
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
    // We can't derive Sub because the inner type doesn't have the expected impls.
    //Sub,
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

    // Shadow DensePolynomial methods, so the types are all `Poly`

    /// Perform a naive `O(n^2)` multiplication of `self` by `other`.
    /// This returns the un-reduced form of the polynomial.
    pub fn naive_mul(&self, other: &Self) -> Self {
        // Deliberately avoid the modular reduction performed by `From`
        // Removing and replacing type wrappers is zero-cost at runtime.
        Self(DensePolynomial::naive_mul(self, other))
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

    /// Returns `self * X^n`, reduced if needed.
    pub fn new_mul_xn(&self, n: usize) -> Self {
        let mut res = Poly::non_canonical_zeroes(n + self.coeffs.len());

        // Copy `self` into the highest coefficients of the polynomial.
        res.coeffs.as_mut_slice()[n..].copy_from_slice(&self.coeffs);

        res.reduce_mod_poly();

        res
    }

    /// Divides `self` by `X^n`, and returns `(newly allocated quotient, self as remainder)`.
    pub fn div_xn(mut self, n: usize) -> (Self, Self) {
        // Make `self` the remainder by splitting off the quotient.
        let quotient = self.coeffs.split_off(n);

        // TODO: `self` keeps the original capacity, is it more efficient to call `shrink_to_fit()` here?
        (Self::from_coefficients_vec(quotient), self)
    }

    /// Divides `self` by `X^n`, and returns a newly allocated `(quotient, remainder)`.
    pub fn new_div_xn(&self, n: usize) -> (Self, Self) {
        // The returned vectors have the exact capacity needed, because they are new allocations.
        let quotient = Poly::from_coefficients_slice(&self.coeffs[n..]);
        let remainder = Poly::from_coefficients_slice(&self.coeffs[..n]);

        (quotient, remainder)
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

    // Private Internal Operations

    /// Returns a new `Poly` filled with `n` zeroes.
    /// This is *not* the canonical form.
    fn non_canonical_zeroes(n: usize) -> Self {
        Self::from_coefficients_vec(vec![Coeff::zero(); n])
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

    /// Read the coefficient at `index`, panicking only when reading a leading zero index above
    /// the maximum degree.
    ///
    /// Use this method instead of `self.coeffs[index]`, to avoid panics when reading leading zero
    /// coefficients.
    /// Use this method instead of `self.get(index)`, to avoid `None` returns when reading leading
    /// zero coefficients.
    ///
    /// # Panics
    ///
    /// Only panics if index is:
    /// - a leading zero coefficient (which is not represented in the underlying data), and
    /// - above [`MAX_POLY_DEGREE`].
    ///
    /// Panics indicate redundant code which should have stopped at the highest non-zero
    /// coefficient. Using `self.coeffs.iter()` is one way to ensure the code only accesses real
    /// coefficients.
    ///
    /// In FHE, zero coefficients will be rare, because the 79 bits of the random coefficient
    /// would all have to be zero. But for performance reasons, we still need to panic if the
    /// coefficient at `index` is zero and above the maximum degree.
    fn index(&self, index: usize) -> &Self::Output {
        match self.coeffs.get(index) {
            Some(coeff) => coeff,
            None => {
                if index <= MAX_POLY_DEGREE {
                    &super::fq::COEFF_ZERO
                } else {
                    panic!("accessed virtual leading zero coefficient: improve performance by stopping at the highest non-zero coefficient")
                }
            }
        }
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
