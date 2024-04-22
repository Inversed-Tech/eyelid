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

impl<const MAX_POLY_DEGREE: usize> Borrow<DensePolynomial<Coeff>> for Poly<MAX_POLY_DEGREE> {
    fn borrow(&self) -> &DensePolynomial<Coeff> {
        &self.0
    }
}

impl<const MAX_POLY_DEGREE: usize> From<Poly<MAX_POLY_DEGREE>>
    for DenseOrSparsePolynomial<'static, Coeff>
{
    fn from(poly: Poly<MAX_POLY_DEGREE>) -> DenseOrSparsePolynomial<'static, Coeff> {
        poly.0.into()
    }
}

impl<'a, const MAX_POLY_DEGREE: usize> From<&'a Poly<MAX_POLY_DEGREE>>
    for DenseOrSparsePolynomial<'a, Coeff>
{
    fn from(poly: &'a Poly<MAX_POLY_DEGREE>) -> DenseOrSparsePolynomial<'a, Coeff> {
        (&poly.0).into()
    }
}

impl<const MAX_POLY_DEGREE: usize> Zero for Poly<MAX_POLY_DEGREE> {
    fn zero() -> Self {
        Self(DensePolynomial { coeffs: vec![] })
    }

    fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }
}

impl<const MAX_POLY_DEGREE: usize> One for Poly<MAX_POLY_DEGREE> {
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

impl<const MAX_POLY_DEGREE: usize> Add<&Poly<MAX_POLY_DEGREE>> for Poly<MAX_POLY_DEGREE> {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self {
        Poly(&self.0 + &rhs.0)
    }
}

impl<const MAX_POLY_DEGREE: usize> Add<Poly<MAX_POLY_DEGREE>> for &Poly<MAX_POLY_DEGREE> {
    type Output = Poly<MAX_POLY_DEGREE>;

    fn add(self, rhs: Poly<MAX_POLY_DEGREE>) -> Self::Output {
        Poly(&self.0 + &rhs.0)
    }
}

impl<'a, 'b, const MAX_POLY_DEGREE: usize> Add<&'a Poly<MAX_POLY_DEGREE>>
    for &'b Poly<MAX_POLY_DEGREE>
{
    type Output = Poly<MAX_POLY_DEGREE>;

    fn add(self, rhs: &'a Poly<MAX_POLY_DEGREE>) -> Self::Output {
        Poly(&self.0 + &rhs.0)
    }
}

impl<const MAX_POLY_DEGREE: usize> Sub for Poly<MAX_POLY_DEGREE> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(&self.0 - &rhs.0)
    }
}

impl<const MAX_POLY_DEGREE: usize> Sub<&Poly<MAX_POLY_DEGREE>> for Poly<MAX_POLY_DEGREE> {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self {
        Poly(&self.0 - &rhs.0)
    }
}

impl<const MAX_POLY_DEGREE: usize> Sub<Poly<MAX_POLY_DEGREE>> for &Poly<MAX_POLY_DEGREE> {
    type Output = Poly<MAX_POLY_DEGREE>;

    fn sub(self, rhs: Poly<MAX_POLY_DEGREE>) -> Self::Output {
        Poly(&self.0 - &rhs.0)
    }
}

impl<'a, 'b, const MAX_POLY_DEGREE: usize> Sub<&'a Poly<MAX_POLY_DEGREE>>
    for &'b Poly<MAX_POLY_DEGREE>
{
    type Output = Poly<MAX_POLY_DEGREE>;

    fn sub(self, rhs: &'a Poly<MAX_POLY_DEGREE>) -> Self::Output {
        Poly(&self.0 - &rhs.0)
    }
}

impl<const MAX_POLY_DEGREE: usize> AddAssign for Poly<MAX_POLY_DEGREE> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += &rhs.0;
    }
}

impl<const MAX_POLY_DEGREE: usize> AddAssign<&Poly<MAX_POLY_DEGREE>> for Poly<MAX_POLY_DEGREE> {
    fn add_assign(&mut self, rhs: &Self) {
        self.0 += &rhs.0;
    }
}

impl<const MAX_POLY_DEGREE: usize> SubAssign for Poly<MAX_POLY_DEGREE> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= &rhs.0;
    }
}

impl<const MAX_POLY_DEGREE: usize> SubAssign<&Poly<MAX_POLY_DEGREE>> for Poly<MAX_POLY_DEGREE> {
    fn sub_assign(&mut self, rhs: &Self) {
        self.0 -= &rhs.0;
    }
}

// Multiplying by a polynomial is a trivial wrapper can't increase the degree, so it is trivial.
impl<const MAX_POLY_DEGREE: usize> Mul<Coeff> for Poly<MAX_POLY_DEGREE> {
    type Output = Self;

    fn mul(self, rhs: Coeff) -> Self {
        Poly(&self.0 * rhs)
    }
}

impl<const MAX_POLY_DEGREE: usize> Mul<Coeff> for &Poly<MAX_POLY_DEGREE> {
    type Output = Poly<MAX_POLY_DEGREE>;

    fn mul(self, rhs: Coeff) -> Self::Output {
        Poly(&self.0 * rhs)
    }
}
