//! Tests for basic polynomial operations.

use ark_ff::{One, Zero};
use ark_poly::{univariate::DenseOrSparsePolynomial, Polynomial};

use crate::primitives::poly::{
    cyclotomic_mul, karatsuba_mul, modular_poly::modulus::poly_modulus, test::gen::rand_poly,
    Coeff, Poly, FULL_RES_POLY_DEGREE,
};

/// Test cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_cyclotomic_mul_rand() {
    let p1: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= FULL_RES_POLY_DEGREE - 1);
    }

    // Xˆ{N-1}, multiplying by it will rotate by N-1 and negate (except the first).
    let xnm1 = Poly::xn(FULL_RES_POLY_DEGREE - 1);

    assert_eq!(xnm1.degree(), FULL_RES_POLY_DEGREE - 1);

    let res = cyclotomic_mul(&p1, &xnm1);
    assert!(res.degree() <= FULL_RES_POLY_DEGREE);

    for i in 0..FULL_RES_POLY_DEGREE - 1 {
        // Negative numbers are automatically converted to canonical
        // representation in the interval [0, MODULUS)
        assert_eq!(res[i], -p1[i + 1]);
    }
    assert_eq!(res[FULL_RES_POLY_DEGREE - 1], p1[0]);

    // Zero leading coefficients aren't stored.
    // `degree()` panics if the leading coefficient is zero anyway.
    assert!(res.degree() < FULL_RES_POLY_DEGREE);
}

/// Test cyclotomic multiplication that results in `X^[MAX_POLY_DEGREE]`.
#[test]
fn test_cyclotomic_mul_max_degree() {
    // X^MAX_POLY_DEGREE
    //
    // Since the degree is equal to MAX_POLY_DEGREE, this is not reduced.
    // But it is in canonical form, because the leading coefficient is non-zero.
    let mut x_max: Poly<FULL_RES_POLY_DEGREE> = Poly::zero();
    x_max[FULL_RES_POLY_DEGREE] = Coeff::one();

    // Manually calculate the reduced representation of X^N as the constant `MODULUS - 1`.
    let x_max = DenseOrSparsePolynomial::from(x_max);
    let (q, x_max) = x_max
        .divide_with_q_and_r(&poly_modulus::<FULL_RES_POLY_DEGREE>())
        .expect("is divisible by X^MAX_POLY_DEGREE");
    let q: Poly<FULL_RES_POLY_DEGREE> = q.into();
    let x_max: Poly<FULL_RES_POLY_DEGREE> = x_max.into();

    assert_eq!(q, Poly::from_coefficients_vec(vec![Coeff::one()]));
    assert_eq!(
        x_max,
        // TODO: should `MODULUS - 1` be a constant?
        Poly::from_coefficients_vec(vec![Coeff::zero() - Coeff::one()]),
    );

    for i in 0..=FULL_RES_POLY_DEGREE {
        // This test is slow, so skip most values.
        if i % 101 != 0
            && ![
                0,
                1,
                FULL_RES_POLY_DEGREE / 2 - 1,
                FULL_RES_POLY_DEGREE / 2,
                FULL_RES_POLY_DEGREE / 2 + 1,
                FULL_RES_POLY_DEGREE - 1,
                FULL_RES_POLY_DEGREE,
            ]
            .contains(&i)
        {
            continue;
        }

        // X^i * X^{MAX_POLY_DEGREE - i} = X^MAX_POLY_DEGREE

        // `p1` and `p2` are automatically reduced if needed.
        let p1 = Poly::xn(i);
        let p2 = Poly::xn(FULL_RES_POLY_DEGREE - i);

        if i == 0 || i == FULL_RES_POLY_DEGREE {
            assert_eq!(p1.degree(), 0);
            assert_eq!(p2.degree(), 0);
        } else {
            assert_eq!(p1.degree() + p2.degree(), FULL_RES_POLY_DEGREE);
        }

        let res = cyclotomic_mul(&p1, &p2);

        // Make sure it's X^N
        assert_eq!(res, x_max);
    }
}

/// Test karatsuba and cyclotomic multiplication of two random polynomials produce the same result.
#[test]
fn test_karatsuba_mul_rand() {
    let p1: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE - 1);
    let p2: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= FULL_RES_POLY_DEGREE - 1);
        assert!(p2.degree() <= FULL_RES_POLY_DEGREE - 1);
    }

    let expected = cyclotomic_mul(&p1, &p2);
    assert!(expected.degree() <= FULL_RES_POLY_DEGREE);
    let res = karatsuba_mul(&p1, &p2);
    assert!(res.degree() <= FULL_RES_POLY_DEGREE);

    assert_eq!(expected, res);
}
