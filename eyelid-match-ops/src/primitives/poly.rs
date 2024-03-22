//! Cyclotomic polynomial operations using ark-poly

use ark_ff::{One, Zero};
use ark_poly::polynomial::{
    univariate::{DenseOrSparsePolynomial, DensePolynomial},
    Polynomial,
};
use lazy_static::lazy_static;

pub mod fq79;
pub mod fq8;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

pub use fq79::{Coeff, MAX_POLY_DEGREE};

/// A modular polynomial with coefficients in [`Coeff`],
/// and maximum degree [`MAX_POLY_DEGREE`].
//
// TODO: replace this with a type wrapper that uses the constant degree MAX_POLY_DEGREE.
pub type Poly = DensePolynomial<Coeff>;

lazy_static! {
    /// The polynomial modulus used for the polynomial field, `X^[MAX_POLY_DEGREE] + 1`.
    /// This means that `X^[MAX_POLY_DEGREE] = -1`.
    pub static ref POLY_MODULUS: DenseOrSparsePolynomial<'static, Coeff> = {
        let mut poly = zero_poly(MAX_POLY_DEGREE);

        poly[MAX_POLY_DEGREE] = Coeff::one();
        poly[0] = Coeff::one();

        assert_eq!(poly.degree(), MAX_POLY_DEGREE);

        poly.into()
    };
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

    let dividend = a.naive_mul(b);

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

    for i in 0..MAX_POLY_DEGREE {
        // In the cyclotomic ring we have that XˆN = -1,
        // therefore all elements from N to 2N-1 are negated.
        if i + MAX_POLY_DEGREE < res.coeffs.len() {
            res[i] = res[i] - res[i + MAX_POLY_DEGREE];
        };
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

    remainder
}
