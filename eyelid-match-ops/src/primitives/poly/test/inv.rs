//! Tests for polynomial inverse.

use crate::primitives::poly::sample;
use ark_ff::{One, Zero};

use crate::primitives::poly::Poly;

#[cfg(test)]
use crate::primitives::poly::{extended_gcd, inverse, FULL_RES_POLY_DEGREE};
#[cfg(test)]
use ark_poly::Polynomial;

#[test]
fn test_inverse() {
    let f = sample::<FULL_RES_POLY_DEGREE>();

    // REMARK: For our parameter choices it is very likely to find
    // the inverse in the first attempt.
    let out = inverse(&f);

    #[cfg(not(tiny_poly))]
    let expect_msg = "unexpected non-invertible large polynomial";
    #[cfg(tiny_poly)]
    let expect_msg = "just checked ok";

    if !cfg!(tiny_poly) || out.is_ok() {
        assert_eq!(f * out.expect(expect_msg), Poly::one());
    } else {
        // For small degree and coefficient modulus, non-invertible polynomials are more likely.

        // Check that `f` isn't invertible
        let (_x, y, _d) = extended_gcd(&Poly::new_unreduced_poly_modulus_slow(), &f);
        let fy = f * y;

        // Since `f` is not invertible, `f * y` can't be `1`.
        assert_ne!(
            fy,
            Poly::one(),
            "incorrect inverse() impl: the inverse of f was y, because f * y == 1"
        );
        // Since all non-zero `Coeff` values *are* invertible in the integer field, `f * y` can't be a constant, either.
        if fy != Poly::zero() {
            assert_ne!(fy.degree(), 0, "incorrect inverse() impl: the inverse of f was y*(c^1), because f * y is a non-zero constant c");
        }
    }
}

#[test]
fn test_edge_cases() {
    // Inverse of one is one
    let one_poly: Poly<FULL_RES_POLY_DEGREE> = Poly::one();
    let mut out = inverse(&one_poly.clone());
    assert_eq!(out, Ok(one_poly.clone()));

    // Inverse of minus one is minus one
    let zero_poly: Poly<FULL_RES_POLY_DEGREE> = Poly::zero();
    let minus_one_poly = zero_poly - one_poly.clone();
    out = inverse(&minus_one_poly.clone());
    assert_eq!(out, Ok(minus_one_poly));

    // Inverse of zero is error
    let zero_poly: Poly<FULL_RES_POLY_DEGREE> = Poly::zero();
    out = inverse(&zero_poly.clone());
    assert!(out.is_err());
}
