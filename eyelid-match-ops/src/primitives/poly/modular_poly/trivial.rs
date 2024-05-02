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

use crate::primitives::poly::{modular_poly::Poly, Coeff, PolyConf};

impl<C: PolyConf> Borrow<DensePolynomial<Coeff>> for Poly<C> {
    fn borrow(&self) -> &DensePolynomial<Coeff> {
        &self.0
    }
}

impl<C: PolyConf> From<Poly<C>> for DenseOrSparsePolynomial<'static, Coeff> {
    fn from(poly: Poly<C>) -> DenseOrSparsePolynomial<'static, Coeff> {
        poly.0.into()
    }
}

impl<'a, C: PolyConf> From<&'a Poly<C>> for DenseOrSparsePolynomial<'a, Coeff> {
    fn from(poly: &'a Poly<C>) -> DenseOrSparsePolynomial<'a, Coeff> {
        (&poly.0).into()
    }
}

impl<C: PolyConf> Zero for Poly<C> {
    fn zero() -> Self {
        Self(DensePolynomial { coeffs: vec![] })
    }

    fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }
}

impl<C: PolyConf> One for Poly<C> {
    fn one() -> Self {
        let mut poly = Self::zero();
        poly[0] = Coeff::one();
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

impl<C: PolyConf> Add<&Poly<C>> for Poly<C> {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self {
        Poly(&self.0 + &rhs.0)
    }
}

impl<C: PolyConf> Add<Poly<C>> for &Poly<C> {
    type Output = Poly<C>;

    fn add(self, rhs: Poly<C>) -> Self::Output {
        Poly(&self.0 + &rhs.0)
    }
}

impl<'a, 'b, C: PolyConf> Add<&'a Poly<C>> for &'b Poly<C> {
    type Output = Poly<C>;

    fn add(self, rhs: &'a Poly<C>) -> Self::Output {
        Poly(&self.0 + &rhs.0)
    }
}

impl<C: PolyConf> Sub for Poly<C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(&self.0 - &rhs.0)
    }
}

impl<C: PolyConf> Sub<&Poly<C>> for Poly<C> {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self {
        Poly(&self.0 - &rhs.0)
    }
}

impl<C: PolyConf> Sub<Poly<C>> for &Poly<C> {
    type Output = Poly<C>;

    fn sub(self, rhs: Poly<C>) -> Self::Output {
        Poly(&self.0 - &rhs.0)
    }
}

impl<'a, 'b, C: PolyConf> Sub<&'a Poly<C>> for &'b Poly<C> {
    type Output = Poly<C>;

    fn sub(self, rhs: &'a Poly<C>) -> Self::Output {
        Poly(&self.0 - &rhs.0)
    }
}

impl<C: PolyConf> AddAssign for Poly<C> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += &rhs.0;
    }
}

impl<C: PolyConf> AddAssign<&Poly<C>> for Poly<C> {
    fn add_assign(&mut self, rhs: &Self) {
        self.0 += &rhs.0;
    }
}

impl<C: PolyConf> SubAssign for Poly<C> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= &rhs.0;
    }
}

impl<C: PolyConf> SubAssign<&Poly<C>> for Poly<C> {
    fn sub_assign(&mut self, rhs: &Self) {
        self.0 -= &rhs.0;
    }
}

// Multiplying by a scalar can't increase the degree, so it is trivial.
impl<C: PolyConf> Mul<Coeff> for Poly<C> {
    type Output = Self;

    fn mul(self, rhs: Coeff) -> Self {
        Poly(&self.0 * rhs)
    }
}

impl<C: PolyConf> Mul<Coeff> for &Poly<C> {
    type Output = Poly<C>;

    fn mul(self, rhs: Coeff) -> Self::Output {
        Poly(&self.0 * rhs)
    }
}
