//! The implementation of a modular polynomial, [`Poly`].
//!
//! This module contains fundamental operations which ensure that the polynomial is always in its canonical form.
//! They should be called after every operation that can create non-canonical polynomials, which can happen when:
//! - the leading coefficient is set to zero, including when the polynomial is split or truncated, or
//! - the degree of the polynomial is increased, for example, during multiplication.

// Optional TODOs:
// - re-implement IndexMut manually, to enforce the canonical form (highest coefficient is non-zero) and modular arithmetic
//   (this can be done by returning a new type with `DerefMut<Target = C::Coeff>``, but it could have performance impacts)
// Trivial:
// - implement Sum manually

use std::{
    marker::PhantomData,
    ops::{Index, IndexMut, Mul},
};

use ark_ff::{One, Zero};
use ark_poly::polynomial::univariate::{
    DenseOrSparsePolynomial, DensePolynomial, SparsePolynomial,
};
use derive_more::{AsRef, Deref, DerefMut, Div, Into, Rem};

use crate::primitives::poly::{mod_poly, mul_poly, new_unreduced_poly_modulus_slow, PolyConf};

pub mod conf;

pub(super) mod inv;
pub(super) mod modulus;
pub(super) mod mul;

mod trivial;

/// A modular polynomial with coefficients in [`PolyConf::Coeff`], and a generic maximum degree
/// [`PolyConf::MAX_POLY_DEGREE`]. The polynomial modulus is `X^MAX_POLY_DEGREE + 1`. Polynomials
/// are always in their canonical, modular reduced form.
///
/// In this canonical form, a polynomial is a list of coefficients from the constant term `X^0`
/// to `X^{MAX_POLY_DEGREE - 1}`, where the highest coefficient is non-zero.
///
/// This canonical form is stored and maintained as follows:
/// - The coefficient of `X^i` is stored at `self[i]`.
/// - `X^MAX_POLY_DEGREE` is reduced to `PolyConf::Coeff::MODULUS - 1`.
/// - Leading zero coefficients are not stored.
/// - If the polynomial is the zero polynomial, the degree is `0`, and there are no coefficients.
///
/// Every operation which can change the degree must call [`Poly::reduce_mod_poly()`].
/// If an operation can create leading zero coefficients, but will never increase the degree, it can call
/// [`Poly::truncate_to_canonical_form()`] instead.
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
    // We can't derive Add or Neg because they add unnecessary bounds on PhantomData
    //Neg,
    //Add,
    // We can't derive Sub because the inner type doesn't have the expected impls.
    //Sub,
    // We don't implement DivAssign and RemAssign, because they have hidden clones.
    Div,
    Rem,
)]
pub struct Poly<C: PolyConf>(
    /// The inner polynomial.
    #[deref]
    #[deref_mut]
    DensePolynomial<C::Coeff>,
    /// A zero-sized marker, which binds the config type to the outer polynomial type.
    PhantomData<C>,
);

impl<C: PolyConf> Poly<C> {
    /// The constant maximum degree of this monomorphized polynomial type.
    pub const N: usize = C::MAX_POLY_DEGREE;

    // Shadow DenseUVPolynomial methods, so we don't have to implement Polynomial and all its supertraits.

    /// Converts the `coeffs` vector into a dense polynomial.
    pub fn from_coefficients_vec(coeffs: Vec<C::Coeff>) -> Self {
        let mut poly = Self(DensePolynomial { coeffs }, PhantomData);

        poly.reduce_mod_poly();

        poly
    }

    /// Converts the `coeffs` slice into a dense polynomial.
    pub fn from_coefficients_slice(coeffs: &[C::Coeff]) -> Self {
        Self::from_coefficients_vec(coeffs.to_vec())
    }

    /// Returns the coefficients of `self` as a mutable slice, skipping any leading zero
    /// coefficients.
    /// `use` the [`ark_poly::DenseUVPolynomial`] trait for the read-only `coeffs()` method.
    ///
    /// After using this low-level accessor, callers must ensure the polynomial is in a canonical
    /// form, by calling either:
    /// - [`Poly::reduce_mod_poly()`], if the degree could have increased, or
    /// - [`Poly::truncate_to_canonical_form()`], if the degree has not increased, but coefficients
    ///   could have been set to zero.
    pub fn coeffs_mut(&mut self) -> &mut [C::Coeff] {
        self.coeffs.as_mut_slice()
    }

    /// Applies `f_zero_to_zero` to the non-zero coefficients of `self`, skipping all zero
    /// coefficients. This excludes leading, trailing, and internal zeroes.
    ///
    /// The polynomial is automatically truncated to its canonical form after the coefficients
    /// are modified.
    ///
    /// # Panics
    ///
    /// If `f_zero_to_zero` does not map zero inputs to zero outputs.
    /// (But it is ok for non-zero inputs to be mapped to zero outputs.)
    pub fn coeffs_modify_non_zero<F>(&mut self, mut f_zero_to_zero: F)
    where
        F: FnMut(&mut C::Coeff),
    {
        assert!({
            let mut z = C::Coeff::zero();
            f_zero_to_zero(&mut z);
            z.is_zero()
        });

        for coeff in self.coeffs_mut() {
            if !coeff.is_zero() {
                f_zero_to_zero(coeff);
            }
        }

        self.truncate_to_canonical_form();
    }

    /// Applies `f` to all the coefficients of `self`, including leading zeroes.
    ///
    /// This method allocates leading zero coefficients, so prefer `coeffs_modify_non_zero()`
    /// where possible.
    ///
    /// The polynomial is automatically truncated to its canonical form after the coefficients
    /// are modified.
    ///
    /// # Panics
    ///
    /// If `f` is not in the canonical reduced form.
    pub fn coeffs_modify_include_zero<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut C::Coeff),
    {
        assert!(self.coeffs.len() <= C::MAX_POLY_DEGREE);

        // Allocate all at once, to avoid allocator churn.
        self.resize_non_canonical_zeroes();

        for i in 0..C::MAX_POLY_DEGREE {
            f(&mut self[i]);
        }

        self.truncate_to_canonical_form();
    }

    /// Maps the non-zero coefficients of `self` to another coefficient type using
    /// `f_zero_to_zero`, and returns the resulting polynomial. This copies trailing and internal
    /// zeroes unmodified, and skips leading zeroes.
    ///
    /// The polynomial is automatically truncated or reduced to its canonical form after the
    /// mapping.
    ///
    /// # Panics
    ///
    /// If `f_zero_to_zero` does not map zero inputs to zero outputs.
    /// (But it is ok for non-zero inputs to be mapped to zero outputs.)
    pub fn map_non_zero<U, F>(&self, mut f_zero_to_zero: F) -> Poly<U>
    where
        U: PolyConf,
        F: FnMut(&C::Coeff) -> U::Coeff,
    {
        assert!({
            let mut z = C::Coeff::zero();
            f_zero_to_zero(&mut z);
            z.is_zero()
        });

        let mut res = Poly::<U>::non_canonical_zeroes(self.coeffs.len());

        for i in 0..self.coeffs.len() {
            if !self[i].is_zero() {
                res[i] = f_zero_to_zero(&self[i]);
            }
        }

        // If the degree is smaller, then the polynomial might need modular reduction.
        res.reduce_mod_poly();

        res
    }

    /// Maps all coefficients of `self` to another coefficient type using `f`, including the
    /// leading zeroes in the *source* polynomial, and returns the resulting polynomial.
    ///
    /// This method allocates leading zero coefficients, so prefer `map_non_zero()`
    /// where possible.
    ///
    /// The polynomial is automatically truncated or reduced to its canonical form after the
    /// mapping.
    ///
    /// # Panics
    ///
    /// If `f` is not in the canonical reduced form.
    pub fn map_include_zero<U, F>(&mut self, mut f: F) -> Poly<U>
    where
        U: PolyConf,
        F: FnMut(&C::Coeff) -> U::Coeff,
    {
        assert!(self.coeffs.len() <= C::MAX_POLY_DEGREE);

        let mut res = Poly::<U>::non_canonical_zeroes(C::MAX_POLY_DEGREE);

        for i in 0..C::MAX_POLY_DEGREE {
            res[i] = f(&self[i]);
        }

        res.reduce_mod_poly();

        res
    }

    /// Maps all coefficients of `self` to an arbitrary type using `f`, including the
    /// leading zeroes, and returns the resulting polynomial.
    ///
    /// This method allocates leading zero coefficients.
    ///
    /// # Panics
    ///
    /// If `f` is not in the canonical reduced form.
    pub fn extract_include_zero<U, F>(&mut self, mut f: F) -> Vec<U>
    where
        F: FnMut(&C::Coeff) -> U,
    {
        assert!(self.coeffs.len() <= C::MAX_POLY_DEGREE);

        let mut res = Vec::with_capacity(C::MAX_POLY_DEGREE);

        for i in 0..C::MAX_POLY_DEGREE {
            res[i] = f(&self[i]);
        }

        res
    }

    // Shadow DensePolynomial methods, so the types are all `Poly`

    /// Perform a naive `O(n^2)` multiplication of `self` by `other`.
    /// This returns the un-reduced form of the polynomial.
    pub fn naive_mul(&self, other: &Self) -> Self {
        // Deliberately avoid the modular reduction performed by `From`
        // Removing and replacing type wrappers is zero-cost at runtime.
        Self(DensePolynomial::naive_mul(self, other), PhantomData)
    }

    // Re-Implement DenseOrSparsePolynomial methods, so the types are all `Poly`

    /// Divide `self`` by another polynomial, and return `(quotient, remainder)`.
    pub fn divide_with_q_and_r(&self, divisor: &Self) -> Option<(Self, Self)> {
        let (quotient, remainder) =
            DenseOrSparsePolynomial::from(self).divide_with_q_and_r(&divisor.into())?;

        Some((quotient.into(), remainder.into()))
    }

    // Efficient Re-Implementations

    /// Returns `X^n` as a polynomial in reduced form.
    pub fn xn(n: usize) -> Self {
        let mut poly = Self::zero();
        poly[n] = C::Coeff::one();

        poly.reduce_mod_poly();

        poly
    }

    /// Multiplies `self` by `X^n`, then reduces if needed.
    pub fn mul_xn(&mut self, n: usize) {
        // Insert `n` zeroes to the lowest coefficients of the polynomial, and shifts the rest up.
        self.coeffs.splice(0..0, vec![C::Coeff::zero(); n]);

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
        let quotient;

        if n <= self.len() {
            quotient = self.coeffs.split_off(n);
            self.truncate_to_canonical_form();
        } else {
            // Poly::zero()
            quotient = Vec::new();
        };

        // TODO: `self` keeps the original capacity, is it more efficient to call `shrink_to_fit()` here?
        (Self::from_coefficients_vec(quotient), self)
    }

    /// Divides `self` by `X^n`, and returns a newly allocated `(quotient, remainder)`.
    pub fn new_div_xn(&self, n: usize) -> (Self, Self) {
        if n <= self.len() {
            // The returned vectors have the exact capacity needed, because they are new allocations.
            let quotient = Poly::from_coefficients_slice(&self.coeffs[n..]);
            let remainder = Poly::from_coefficients_slice(&self.coeffs[..n]);

            (quotient, remainder)
        } else {
            (Poly::zero(), self.clone())
        }
    }

    // Basic Internal Operations

    /// Returns the primitive inverse of this polynomial in the cyclotomic ring, if it exists.
    /// Otherwise, returns an error.
    pub fn inverse(&self) -> Result<Self, &'static str> {
        inv::inverse(self)
    }

    /// Constructs and returns a new polynomial modulus used for the polynomial field, `X^[C::MAX_POLY_DEGREE] + 1`.
    /// This is the canonical but un-reduced form of the modulus, because the reduced form is the zero polynomial.
    pub fn new_unreduced_poly_modulus_slow() -> Self {
        new_unreduced_poly_modulus_slow()
    }

    /// Multiplies two polynomials, and returns the result in reduced form.
    ///
    /// This operation can be called using the `*` operator, this method is only needed to disambiguate.
    pub fn mul_reduce(&self, rhs: &Self) -> Self {
        mul_poly(self, rhs)
    }

    /// Reduce this polynomial so it is less than the polynomial modulus.
    /// This also ensures its degree is less than [[`PolyConf::MAX_POLY_DEGREE`]](Self::N).
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
        while self.coeffs.last() == Some(&C::Coeff::zero()) {
            self.coeffs.pop();
        }
    }

    // Private Internal Operations

    /// Returns a new `Poly` filled with `n` zeroes.
    /// This is *not* the canonical form.
    pub(crate) fn non_canonical_zeroes(n: usize) -> Self {
        Self(
            DensePolynomial {
                coeffs: vec![C::Coeff::zero(); n],
            },
            PhantomData,
        )
    }

    /// Extends this polynomial with zeroes, up to [`C::MAX_POLY_DEGREE`](PolyConf::MAX_POLY_DEGREE).
    /// The extended polynomial is *not guaranteed* to be in the canonical form.
    pub(crate) fn resize_non_canonical_zeroes(&mut self) {
        self.coeffs.resize(C::MAX_POLY_DEGREE, C::Coeff::zero());
    }
}

impl<C: PolyConf> From<DensePolynomial<C::Coeff>> for Poly<C> {
    fn from(poly: DensePolynomial<C::Coeff>) -> Self {
        let mut poly = Self(poly, PhantomData);
        poly.reduce_mod_poly();
        poly
    }
}

impl<C: PolyConf> From<&DensePolynomial<C::Coeff>> for Poly<C> {
    fn from(poly: &DensePolynomial<C::Coeff>) -> Self {
        let mut poly = Self(poly.clone(), PhantomData);
        poly.reduce_mod_poly();
        poly
    }
}

// These are less likely to be called, so it's ok to have sub-optimal performance.
impl<C: PolyConf> From<SparsePolynomial<C::Coeff>> for Poly<C> {
    fn from(poly: SparsePolynomial<C::Coeff>) -> Self {
        DensePolynomial::from(poly).into()
    }
}

impl<C: PolyConf> From<&SparsePolynomial<C::Coeff>> for Poly<C> {
    fn from(poly: &SparsePolynomial<C::Coeff>) -> Self {
        DensePolynomial::from(poly.clone()).into()
    }
}

impl<'a, C: PolyConf> From<DenseOrSparsePolynomial<'a, C::Coeff>> for Poly<C> {
    fn from(poly: DenseOrSparsePolynomial<'a, C::Coeff>) -> Self {
        DensePolynomial::from(poly).into()
    }
}

impl<'a, C: PolyConf> From<&DenseOrSparsePolynomial<'a, C::Coeff>> for Poly<C> {
    fn from(poly: &DenseOrSparsePolynomial<'a, C::Coeff>) -> Self {
        DensePolynomial::from(poly.clone()).into()
    }
}

impl<C: PolyConf> Index<usize> for Poly<C> {
    type Output = C::Coeff;

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
    /// - above [[`PolyConf::MAX_POLY_DEGREE`]](Self::N).
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
                if index <= C::MAX_POLY_DEGREE {
                    C::coeff_zero()
                } else {
                    panic!("accessed virtual leading zero coefficient: improve performance by stopping at the highest non-zero coefficient")
                }
            }
        }
    }
}

impl<C: PolyConf> IndexMut<usize> for Poly<C> {
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
            self.coeffs.resize(index + 1, C::Coeff::zero());
        }

        self.coeffs.get_mut(index).expect("just resized")
    }
}

// We don't implement operators for SparsePolynomial or DenseOrSparsePolynomial, they are rare and can use .into() to convert first.
impl<C: PolyConf> Mul for Poly<C> {
    type Output = Self;

    /// Multiplies then reduces by the polynomial modulus.
    fn mul(self, rhs: Self) -> Self {
        mul_poly(&self, &rhs)
    }
}

impl<C: PolyConf> Mul<&Poly<C>> for Poly<C> {
    type Output = Self;

    /// Multiplies then reduces by the polynomial modulus.
    fn mul(self, rhs: &Self) -> Self {
        mul_poly(&self, rhs)
    }
}

impl<C: PolyConf> Mul for &Poly<C> {
    type Output = Poly<C>;

    /// Multiplies then reduces by the polynomial modulus.
    fn mul(self, rhs: Self) -> Self::Output {
        mul_poly(self, rhs)
    }
}

impl<C: PolyConf> Mul<Poly<C>> for &Poly<C> {
    type Output = Poly<C>;

    /// Multiplies then reduces by the polynomial modulus.
    fn mul(self, rhs: Poly<C>) -> Self::Output {
        mul_poly(self, &rhs)
    }
}

impl<C: PolyConf> Mul<DensePolynomial<C::Coeff>> for Poly<C> {
    type Output = Self;

    /// Multiplies then reduces by the polynomial modulus.
    fn mul(self, rhs: DensePolynomial<C::Coeff>) -> Self {
        mul_poly(&self, &Self(rhs, PhantomData))
    }
}

// TODO: if we need this method, remove the clone() using unsafe code
#[cfg(inefficient)]
impl<C: PolyConf> Mul<&DensePolynomial<C::Coeff>> for Poly<C> {
    type Output = Self;

    /// Multiplies then reduces by the polynomial modulus.
    fn mul(self, rhs: &DensePolynomial<C::Coeff>) -> Self {
        mul_poly(&self, &Self(rhs.clone()), PhantomData)
    }
}
