//! Cyclotomic polynomial operations using ark-poly

use ark_ff::{One, Zero};
use ark_poly::polynomial::{
    univariate::{DenseOrSparsePolynomial, DensePolynomial},
    Polynomial,
};
use lazy_static::lazy_static;
use std::ops::Sub;

pub mod fq79;
pub mod fq8;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

pub use fq79::{Coeff, MAX_POLY_DEGREE};
// Temporarily switch to this tiny field to make test errors easier to debug.
//pub use fq8::{Coeff, MAX_POLY_DEGREE};

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

/// Returns a Boolean indicating if the input is equal or not to the additive
/// identity in the polynomial ring
pub fn is_zero_poly(a: Poly) -> bool {
    let mut poly = Poly::zero();
    poly.coeffs = vec![Coeff::zero(); MAX_POLY_DEGREE + 1];
    poly == a
}

/// Returns the multiplicative element of the polynomial ring.
pub fn one_poly(degree: usize) -> Poly {
    assert!(degree <= MAX_POLY_DEGREE);

    let mut poly = Poly::zero();
    poly.coeffs = vec![Coeff::zero(); degree + 1];
    poly.coeffs[0] = Coeff::one();
    poly
}


/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn cyclotomic_mul(a: &Poly, b: &Poly) -> Poly {
    // TODO: change these assertions to debug_assert!() to avoid panics in production code.
    dbg!("bug after");
    assert!(a.degree() <= MAX_POLY_DEGREE);
    dbg!("bug before");
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

    remainder
}

/// Returns polynomials x, y, d such that a.x + a.y = d.
/// When d=1 we have that x is the multiplicative inverse of a.
pub fn extended_gcd(a: &Poly, b: Poly) -> (Poly, Poly, Poly) {
    // Invariant a.xi + b.yi = ri

    // init with x0=1, y0=0, r0=a
    let mut x_prev = one_poly(MAX_POLY_DEGREE);
    let mut y_prev = zero_poly(MAX_POLY_DEGREE);
    let ri_prev = a.clone();
    // next:     x1=0, y1=1, r1=b
    // FIXME: we need a way to create the zero polynomial, whose degree is zero
    // But right now if we use the zero_poly function, the program will panic when
    // degree() is called inside cyclotomic_mul
    // See Issue #13
    let mut x_cur = zero_poly(0);
    let mut y_cur = one_poly(0);
    let ri_cur = b.clone();

    let mut dividend: DenseOrSparsePolynomial<'_, _> = ri_prev.clone().into();
    let (mut ri_cur, mut q) = dividend
        .divide_with_q_and_r(&ri_cur.into())
        .expect("POLY_MODULUS is not zero");
    // xi+1 = xi-1 - q.xi
    let mut x_aux = x_cur.clone();
    x_cur = x_prev.sub(&cyclotomic_mul(&q, &x_cur));
    x_prev = x_aux;
    // yi+1 = yi-1 - q.yi
    let mut y_aux = y_cur.clone();
    y_cur = y_prev.sub(&cyclotomic_mul(&q, &y_cur));
    y_prev = y_aux;
    // loop until ri_cur = 0
    while !is_zero_poly(ri_cur.clone()) {
        dividend = ri_prev.clone().into();
        (ri_cur, q) = dividend
            .divide_with_q_and_r(&*POLY_MODULUS)
            .expect("POLY_MODULUS is not zero");
        // xi+1 = xi-1 - q.xi
        x_aux = x_cur.clone();
        x_cur = x_prev.sub(&cyclotomic_mul(&q, &x_cur));
        x_prev = x_aux;
        // yi+1 = yi-1 - q.yi
        y_aux = y_cur.clone();
        y_cur = y_prev.sub(&cyclotomic_mul(&q, &y_cur));
        y_prev = y_aux;
    }
    (x_cur, y_cur, ri_cur)
}
