//! Trivial operations on [`Poly`].
//!
//! This module implements trivial polynomial operations, which just forward to the underlying [`DensePolynomial`].
//! The derives on [`Poly`] are also trivial operations.

use std::{
    borrow::Borrow,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

use ark_ff::{One, Zero};
use ark_poly::polynomial::univariate::{DenseOrSparsePolynomial, DensePolynomial};

use crate::primitives::poly::modular_poly::{Coeff, Poly};

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

// Poly + Poly and similar are provided by the derive

// TODO:
// Some missing truncate_leading_zeroes() can cause a panic in degree():
// <https://github.com/Inversed-Tech/eyelid/issues/43>

impl Add<&Poly> for Poly {
    type Output = Self;

    fn add(self, rhs: &Self) -> Poly {
        Poly(&self.0 + &rhs.0)
    }
}

impl Add<Poly> for &Poly {
    type Output = Poly;

    fn add(self, rhs: Poly) -> Poly {
        Poly(&self.0 + &rhs.0)
    }
}

impl<'a, 'b> Add<&'a Poly> for &'b Poly {
    type Output = Poly;

    fn add(self, rhs: &'a Poly) -> Poly {
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

impl Sub<Poly> for &Poly {
    type Output = Poly;

    fn sub(self, rhs: Poly) -> Poly {
        Poly(&self.0 - &rhs.0)
    }
}

impl<'a, 'b> Sub<&'a Poly> for &'b Poly {
    type Output = Poly;

    fn sub(self, rhs: &'a Poly) -> Poly {
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

// Multiplying by a scalar can't increase the degree, so it is trivial.
impl Mul<Coeff> for Poly {
    type Output = Self;

    fn mul(self, rhs: Coeff) -> Self {
        Poly(&self.0 * rhs)
    }
}

impl Mul<Coeff> for &Poly {
    type Output = Poly;

    fn mul(self, rhs: Coeff) -> Poly {
        Poly(&self.0 * rhs)
    }
}
