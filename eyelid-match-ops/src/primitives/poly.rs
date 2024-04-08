//! Cyclotomic polynomial operations using ark-poly

use std::{
    borrow::Borrow,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

use ark_ff::{One, Zero};
use ark_poly::{
    polynomial::{
        univariate::{DenseOrSparsePolynomial, DensePolynomial},
        Polynomial,
    },
    DenseUVPolynomial,
};
use derive_more::{
    Add, AsRef, Constructor, Deref, DerefMut, Div, DivAssign, From, Index, IndexMut, Into, Mul,
    MulAssign, Neg, Rem, RemAssign,
};
use lazy_static::lazy_static;
use rand::Rng;

pub use fq::{Coeff, MAX_POLY_DEGREE};

pub mod fq;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

/// Minimum degree for recursive Karatsuba calls
pub const MIN_KARATSUBA_REC_DEGREE: usize = 32; // TODO: fine tune

lazy_static! {
    /// The polynomial modulus used for the polynomial field, `X^[MAX_POLY_DEGREE] + 1`.
    /// This means that `X^[MAX_POLY_DEGREE] = -1`.
    pub static ref POLY_MODULUS: DenseOrSparsePolynomial<'static, Coeff> = {
        let mut poly = zero_poly(MAX_POLY_DEGREE);

        poly[MAX_POLY_DEGREE] = Coeff::one();
        poly[0] = Coeff::one();

        assert_eq!(poly.degree(), MAX_POLY_DEGREE);

        poly.0.into()
    };
}

/// A modular polynomial with coefficients in [`Coeff`], and maximum degree [`MAX_POLY_DEGREE`].
//
// TODO:
// - move Poly into its own file
// - enforce the constant degree MAX_POLY_DEGREE
// - re-implement Index and IndexMut manually, to enforce the canonical form (highest coefficient is non-zero) and modular arithmetic
// - re-implement Mul and MulAssign manually, to enforce modular arithmetic by POLY_MODULUS (Add, Sub, Div, Rem, and Neg can't increase the degree)
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

/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn cyclotomic_mul(a: &Poly, b: &Poly) -> Poly {
    // TODO: change these assertions to debug_assert!() to avoid panics in production code.
    assert!(a.degree() <= MAX_POLY_DEGREE);
    assert!(b.degree() <= MAX_POLY_DEGREE);

    let dividend: Poly = a.naive_mul(b).into();

    // Use the fastest benchmark between mod_poly_manual() and mod_poly_ark() here,
    // and debug_assert_eq!() the other one.
    let res = mod_poly_manual(&dividend);
    debug_assert_eq!(res, mod_poly_ark(&dividend));

    assert!(res.degree() <= MAX_POLY_DEGREE);

    res
}

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This is a manual implementation.
pub fn mod_poly_manual(dividend: &Poly) -> Poly {
    let mut res = dividend.clone();

    let mut i = MAX_POLY_DEGREE;
    while i < res.coeffs.len() {
        // In the cyclotomic ring we have that XˆN = -1,
        // therefore all elements from N to 2N-1 are negated.

        let q = i / MAX_POLY_DEGREE;
        let r = i % MAX_POLY_DEGREE;
        if q % 2 == 1 {
            res[r] = res[r] - res[i];
        } else {
            res[r] = res[r] + res[i];
        }
        i += 1;
    }

    // These elements have already been negated and summed above.
    res.coeffs.truncate(MAX_POLY_DEGREE);

    // Leading elements might be zero, so make sure the polynomial is in the canonical form.
    while res.coeffs.last() == Some(&Coeff::zero()) {
        res.coeffs.pop();
    }

    res
}

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This uses an [`ark-poly`] library implementation.
pub fn mod_poly_ark(dividend: &Poly) -> Poly {
    let dividend: DenseOrSparsePolynomial<'_, _> = dividend.into();

    let (_quotient, remainder) = dividend
        .divide_with_q_and_r(&*POLY_MODULUS)
        .expect("POLY_MODULUS is not zero");

    remainder.into()
}

/// Returns `a * b` followed by reduction mod `XˆN + 1` using recursive Karatsuba method.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn karatsuba_mul(a: &Poly, b: &Poly) -> Poly {
    let mut res;
    let n = a.degree() + 1;

    // if a or b has degree less than min, condition is true
    let cond_a = a.degree() + 1 < MIN_KARATSUBA_REC_DEGREE;
    let cond_b = b.degree() + 1 < MIN_KARATSUBA_REC_DEGREE;
    let rec_cond = cond_a || cond_b;
    if rec_cond {
        // If degree is less than the recursion minimum, just use the naive version
        res = cyclotomic_mul(a, b);
    } else {
        // Otherwise recursively call for al.bl and ar.br
        let (al, ar) = poly_split(a);
        let (bl, br) = poly_split(b);
        let albl = karatsuba_mul(&al, &bl);
        let arbr = karatsuba_mul(&ar, &br);
        let alpar = al.add(ar);
        let blpbr = bl.add(br);
        // Compute y = (al + ar).(bl + br)
        let y = karatsuba_mul(&alpar, &blpbr);
        // Compute res = al.bl + (y - al.bl - ar.br)xˆn/2 + (ar.br)x^n
        res = y.clone();
        res = res.sub(&albl);
        res = res.sub(&arbr);
        let halfn = n / 2;
        let mut xnb2 = zero_poly(halfn);
        xnb2.coeffs[halfn] = Coeff::one();
        res = cyclotomic_mul(&res.clone(), &xnb2);
        res = res.add(albl);
        if n >= MAX_POLY_DEGREE {
            // negate ar.br if n is equal to the max degree (edge case)
            res = res.sub(&arbr);
        } else {
            // Otherwise proceed as usual
            let mut xn = zero_poly(n);
            xn.coeffs[n] = Coeff::one();
            let aux = cyclotomic_mul(&arbr, &xn);
            res = res.add(aux);
        }
    };
    res
}

/// Split the polynomial into left and right parts.
pub fn poly_split(a: &Poly) -> (Poly, Poly) {
    // TODO: review performance
    let n = a.degree() + 1;
    let halfn = n / 2;
    let mut al = a.clone();
    let ar = al.coeffs.split_off(halfn);
    (al, DensePolynomial { coeffs: ar }.into())
}
