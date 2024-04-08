//! The base implementation of a modular polynomial, [`Poly`].
//!
//! This module contains the transparent operations, which just forward to the underlying [`DensePolynomial`].

use std::{
    borrow::Borrow,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

use ark_ff::{One, Zero};
use ark_poly::{
    polynomial::univariate::{DenseOrSparsePolynomial, DensePolynomial},
    DenseUVPolynomial,
};
use derive_more::{
    Add, AsRef, Constructor, Deref, DerefMut, Div, DivAssign, From, Index, IndexMut, Into, Mul,
    MulAssign, Neg, Rem, RemAssign,
};

use super::Coeff;

#[cfg(any(test, feature = "benchmark"))]
use rand::Rng;

/// A modular polynomial with coefficients in [`Coeff`], and maximum degree [`MAX_POLY_DEGREE`].
//
// TODO:
// Optional:
// - implement Sum manually
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

impl Poly {
    // Partial implementation of DenseUVPolynomial

    /// Converts `coeffs` into a dense polynomial.
    pub fn from_coefficients_vec(coeffs: Vec<Coeff>) -> Self {
        Self(DensePolynomial { coeffs })
    }

    /// Returns a random polynomial with degree `d`.
    /// Only for use in tests and benchmarks.
    #[cfg(any(test, feature = "benchmark"))]
    pub fn rand<R: Rng>(d: usize, rng: &mut R) -> Self {
        DensePolynomial::rand(d, rng).into()
    }
}

impl Borrow<DensePolynomial<Coeff>> for Poly {
    fn borrow(&self) -> &DensePolynomial<Coeff> {
        &self.0
    }
}

impl From<Poly> for DenseOrSparsePolynomial<'static, Coeff> {
    fn from(poly: Poly) -> DenseOrSparsePolynomial<'static, Coeff> {
        poly.0.into()
    }
}

impl<'a> From<&'a Poly> for DenseOrSparsePolynomial<'a, Coeff> {
    fn from(poly: &'a Poly) -> DenseOrSparsePolynomial<'a, Coeff> {
        (&poly.0).into()
    }
}

impl Zero for Poly {
    fn zero() -> Self {
        Self(DensePolynomial { coeffs: vec![] })
    }

    fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }
}

impl One for Poly {
    fn one() -> Self {
        let mut poly = Self::zero();
        poly.coeffs[0] = Coeff::one();
        poly
    }

    fn set_one(&mut self) {
        self.coeffs = vec![Coeff::one()];
    }

    fn is_one(&self) -> bool {
        self.coeffs == vec![Coeff::one()]
    }
}

// Poly + Poly is provided by the derive

impl Add<&Poly> for Poly {
    type Output = Self;

    fn add(self, rhs: &Self) -> Poly {
        Poly(&self.0 + &rhs.0)
    }
}

impl Sub for Poly {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(&self.0 - &rhs.0)
    }
}

impl Sub<&Poly> for Poly {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Poly {
        Poly(&self.0 - &rhs.0)
    }
}

impl AddAssign<Poly> for Poly {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += &rhs.0;
    }
}

impl AddAssign<&Poly> for Poly {
    fn add_assign(&mut self, rhs: &Self) {
        self.0 += &rhs.0;
    }
}

impl SubAssign<Poly> for Poly {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= &rhs.0;
    }
}

impl SubAssign<&Poly> for Poly {
    fn sub_assign(&mut self, rhs: &Self) {
        self.0 -= &rhs.0;
    }
}

impl Mul for Poly {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(&self.0 * &rhs.0)
    }
}
