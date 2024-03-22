//! Cyclotomic polynomial operations using ark-poly

use ark_ff::{Fp128, MontBackend, MontConfig, One, Zero};
use ark_poly::polynomial::{
    univariate::{DenseOrSparsePolynomial, DensePolynomial},
    Polynomial,
};
use lazy_static::lazy_static;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

/// The maximum exponent in the polynomial.
pub const MAX_POLY_DEGREE: usize = 2048;

/// The configuration of the modular field used for polynomial coefficients.
#[derive(MontConfig)]
#[modulus = "93309596432438992665667"]
#[generator = "5"]
pub struct Fq79Config;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
/// t = 2ˆ15, q = 2ˆ79, N = 2048
//
// Sage commands:
// random_prime(2**79)
// 93309596432438992665667
// ff = GF(93309596432438992665667)
// ff.multiplicative_generator()
// 5
//
// We could also consider generating primes dynamically, but this could impact performance.
pub type Fq79 = Fp128<MontBackend<Fq79Config, 2>>;

/// A modular polynomial with coefficients in [`Fq79`],
/// and maximum degree [`MAX_POLY_DEGREE`].
//
// TODO: replace this with a type wrapper that uses the constant degree above.
pub type Poly = DensePolynomial<Fq79>;

lazy_static! {
    /// The polynomial modulus used for the polynomial field, `X^[MAX_POLY_DEGREE] + 1`.
    /// This means that `X^[MAX_POLY_DEGREE] = -1`.
    pub static ref POLY_MODULUS: DenseOrSparsePolynomial<'static, Fq79> = {
        let mut poly = zero_poly(MAX_POLY_DEGREE);

        poly[MAX_POLY_DEGREE] = Fq79::one();
        poly[0] = Fq79::one();

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
    poly.coeffs = vec![Fq79::zero(); degree + 1];
    poly
}

/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn cyclotomic_mul(a: &Poly, b: &Poly) -> Poly {
    // TODO: change these assertions to debug_assert!() to avoid panics in production code.
    assert!(a.degree() <= MAX_POLY_DEGREE);
    assert!(b.degree() <= MAX_POLY_DEGREE);

    let mut res = a.naive_mul(b);

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
    while res.coeffs.last() == Some(&Fq79::zero()) {
        res.coeffs.pop();
    }

    assert!(res.degree() <= MAX_POLY_DEGREE);

    res
}
