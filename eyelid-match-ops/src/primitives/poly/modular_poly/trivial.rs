//! Trivial operations on [`Poly`].
//!
//! This module implements trivial polynomial operations, which just forward to the underlying [`DensePolynomial`].
//! The derives on [`Poly`] are also trivial operations.

use std::{
    borrow::Borrow,
    marker::PhantomData,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

use ark_ff::{One, Zero};
use ark_poly::polynomial::univariate::{DenseOrSparsePolynomial, DensePolynomial};

use crate::primitives::poly::{modular_poly::Poly, PolyConf};

impl<C: PolyConf> Borrow<DensePolynomial<C::Coeff>> for Poly<C> {
    fn borrow(&self) -> &DensePolynomial<C::Coeff> {
        &self.0
    }
}

impl<C: PolyConf> From<Poly<C>> for DenseOrSparsePolynomial<'static, C::Coeff> {
    fn from(poly: Poly<C>) -> DenseOrSparsePolynomial<'static, C::Coeff> {
        poly.0.into()
    }
}

impl<'a, C: PolyConf> From<&'a Poly<C>> for DenseOrSparsePolynomial<'a, C::Coeff> {
    fn from(poly: &'a Poly<C>) -> DenseOrSparsePolynomial<'a, C::Coeff> {
        (&poly.0).into()
    }
}

impl<C: PolyConf> Zero for Poly<C> {
    fn zero() -> Self {
        Self(DensePolynomial { coeffs: vec![] }, PhantomData)
    }

    fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }
}

impl<C: PolyConf> One for Poly<C> {
    fn one() -> Self {
        let mut poly = Self::zero();
        poly[0] = C::Coeff::one();
        poly
    }

    fn set_one(&mut self) {
        self.coeffs = vec![C::Coeff::one()];
    }

    fn is_one(&self) -> bool {
        self.coeffs == vec![C::Coeff::one()]
    }
}

// Poly / Poly and Poly % Poly are provided by the derives

// TODO:
// Some missing truncate_leading_zeroes() can cause a panic in degree():
// <https://github.com/Inversed-Tech/eyelid/issues/43>

impl<C: PolyConf> Neg for Poly<C> {
    type Output = Self;

    fn neg(self) -> Self {
        Poly(-self.0, PhantomData)
    }
}

impl<C: PolyConf> Add<Poly<C>> for Poly<C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Poly(&self.0 + &rhs.0, PhantomData)
    }
}

impl<C: PolyConf> Add<&Poly<C>> for Poly<C> {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self {
        Poly(&self.0 + &rhs.0, PhantomData)
    }
}

impl<C: PolyConf> Add<Poly<C>> for &Poly<C> {
    type Output = Poly<C>;

    fn add(self, rhs: Poly<C>) -> Self::Output {
        Poly(&self.0 + &rhs.0, PhantomData)
    }
}

impl<'a, 'b, C: PolyConf> Add<&'a Poly<C>> for &'b Poly<C> {
    type Output = Poly<C>;

    fn add(self, rhs: &'a Poly<C>) -> Self::Output {
        Poly(&self.0 + &rhs.0, PhantomData)
    }
}

impl<C: PolyConf> Sub for Poly<C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(&self.0 - &rhs.0, PhantomData)
    }
}

impl<C: PolyConf> Sub<&Poly<C>> for Poly<C> {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self {
        Poly(&self.0 - &rhs.0, PhantomData)
    }
}

impl<C: PolyConf> Sub<Poly<C>> for &Poly<C> {
    type Output = Poly<C>;

    fn sub(self, rhs: Poly<C>) -> Self::Output {
        Poly(&self.0 - &rhs.0, PhantomData)
    }
}

impl<'a, 'b, C: PolyConf> Sub<&'a Poly<C>> for &'b Poly<C> {
    type Output = Poly<C>;

    fn sub(self, rhs: &'a Poly<C>) -> Self::Output {
        Poly(&self.0 - &rhs.0, PhantomData)
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

// `Mul` by a scalar conflicts with multiplying by a polynomial.
// Use `MulAssign` or `*=` instead.
