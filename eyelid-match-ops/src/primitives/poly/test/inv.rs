//! Tests for polynomial inverse.

use ark_ff::{One, Zero};

use crate::primitives::{
    poly::{
        modular_poly::inv::{extended_gcd, inverse},
        test::gen::rand_poly,
        Poly, PolyConf, TestRes,
    },
    yashe::{Yashe, YasheParams},
};

#[cfg(test)]
use crate::primitives::poly::FULL_RES_POLY_DEGREE;
#[cfg(test)]
use ark_poly::Polynomial;

fn inverse_test_helper<C: PolyConf>(f: &Poly<C>) {
    // REMARK: For our parameter choices it is very likely to find
    // the inverse in the first attempt.
    let out = inverse(f);

    #[cfg(not(tiny_poly))]
    let expect_msg = "unexpected non-invertible large polynomial";
    #[cfg(tiny_poly)]
    let expect_msg = "just checked ok";

    if !cfg!(tiny_poly) || out.is_ok() {
        assert_eq!(f * out.expect(expect_msg), Poly::one());
    } else {
        // For small degree and coefficient modulus, non-invertible polynomials are more likely.

        // Check that `f` isn't invertible
        let (_x, y, _d) = extended_gcd(&Poly::new_unreduced_poly_modulus_slow(), f);
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
fn test_key_generation_and_inverse() {
    let mut rng = rand::thread_rng();

    let params = YasheParams {
        t: 1024,
        delta: 3.2,
    };
    let ctx: Yashe<TestRes> = Yashe::new(params);
    let f = ctx.sample_gaussian(&mut rng);

    // REMARK: For our parameter choices it is very likely to find
    // the inverse in the first attempt.
    inverse_test_helper(&f);
}

#[test]
fn test_inverse_with_random_coefficients() {
    let f: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);
    inverse_test_helper(&f);
}

#[test]
fn test_edge_cases() {
    // Inverse of one is one
    let one_poly: Poly<TestRes> = Poly::one();
    let mut out = inverse(&one_poly.clone());
    assert_eq!(out, Ok(one_poly.clone()));

    // Inverse of minus one is minus one
    let zero_poly: Poly<TestRes> = Poly::zero();
    let minus_one_poly = zero_poly - one_poly.clone();
    out = inverse(&minus_one_poly.clone());
    assert_eq!(out, Ok(minus_one_poly));

    // Inverse of zero is error
    let zero_poly: Poly<TestRes> = Poly::zero();
    out = inverse(&zero_poly.clone());
    assert!(out.is_err());
}
