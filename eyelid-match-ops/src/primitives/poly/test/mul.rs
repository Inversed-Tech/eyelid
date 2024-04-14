//! Tests for basic polynomial operations.

use ark_ff::{One, Zero};
use ark_poly::{univariate::DenseOrSparsePolynomial, Polynomial};

use crate::primitives::poly::{
    cyclotomic_mul, karatsuba_mul, test::gen::rand_poly, Coeff, Poly, MAX_POLY_DEGREE, POLY_MODULUS,
};

/// Test cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_cyclotomic_mul_rand() {
    let p1 = rand_poly(MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MAX_POLY_DEGREE - 1);
    }

    // Xˆ{N-1}, multiplying by it will rotate by N-1 and negate (except the first).
    let xnm1 = Poly::xn(MAX_POLY_DEGREE - 1);

    assert_eq!(xnm1.degree(), MAX_POLY_DEGREE - 1);

    let res = cyclotomic_mul(&p1, &xnm1);
    assert!(res.degree() <= MAX_POLY_DEGREE);

    for i in 0..MAX_POLY_DEGREE - 1 {
        // Negative numbers are automatically converted to canonical
        // representation in the interval [0, MODULUS)
        assert_eq!(res[i], -p1[i + 1]);
    }
    assert_eq!(res[MAX_POLY_DEGREE - 1], p1[0]);

    // Zero leading coefficients aren't stored.
    // `degree()` panics if the leading coefficient is zero anyway.
    assert!(res.degree() < MAX_POLY_DEGREE);
}

/// Test cyclotomic multiplication that results in `X^[MAX_POLY_DEGREE]`.
#[test]
fn test_cyclotomic_mul_max_degree() {
    // X^MAX_POLY_DEGREE
    //
    // Since the degree is equal to MAX_POLY_DEGREE, this is not reduced.
    // But it is in canonical form, because the leading coefficient is non-zero.
    let mut x_max = Poly::zero();
    x_max[MAX_POLY_DEGREE] = Coeff::one();

    // Manually calculate the reduced representation of X^N as the constant `MODULUS - 1`.
    let x_max = DenseOrSparsePolynomial::from(x_max);
    let (q, x_max) = x_max
        .divide_with_q_and_r(&*POLY_MODULUS)
        .expect("is divisible by X^MAX_POLY_DEGREE");
    let q: Poly = q.into();
    let x_max: Poly = x_max.into();

    assert_eq!(q, Poly::from_coefficients_vec(vec![Coeff::one()]));
    assert_eq!(
        x_max,
        // TODO: should `MODULUS - 1` be a constant?
        Poly::from_coefficients_vec(vec![Coeff::zero() - Coeff::one()]),
    );

    for i in 0..=MAX_POLY_DEGREE {
        // This test is slow, so skip most values.
        if i % 101 != 0
            && ![
                0,
                1,
                MAX_POLY_DEGREE / 2 - 1,
                MAX_POLY_DEGREE / 2,
                MAX_POLY_DEGREE / 2 + 1,
                MAX_POLY_DEGREE - 1,
                MAX_POLY_DEGREE,
            ]
            .contains(&i)
        {
            continue;
        }

        // X^i * X^{MAX_POLY_DEGREE - i} = X^MAX_POLY_DEGREE

        // `p1` and `p2` are automatically reduced if needed.
        let p1 = Poly::xn(i);
        let p2 = Poly::xn(MAX_POLY_DEGREE - i);

        if i == 0 || i == MAX_POLY_DEGREE {
            assert_eq!(p1.degree(), 0);
            assert_eq!(p2.degree(), 0);
        } else {
            assert_eq!(p1.degree() + p2.degree(), MAX_POLY_DEGREE);
        }

        let res = cyclotomic_mul(&p1, &p2);

        // Make sure it's X^N
        assert_eq!(res, x_max);
    }
}

/// Test karatsuba and cyclotomic multiplication of two random polynomials produce the same result.
#[test]
fn test_karatsuba_mul_rand() {
    let p1 = rand_poly(MAX_POLY_DEGREE - 1);
    let p2 = rand_poly(MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MAX_POLY_DEGREE - 1);
        assert!(p2.degree() <= MAX_POLY_DEGREE - 1);
    }

    let expected = cyclotomic_mul(&p1, &p2);
    assert!(expected.degree() <= MAX_POLY_DEGREE);
    let res = karatsuba_mul(&p1, &p2);
    assert!(res.degree() <= MAX_POLY_DEGREE);

    assert_eq!(expected, res);
}

/// Test flat Karatsuba cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_flat_karatsuba_mul_rand() {
    let p1 = rand_poly(MAX_POLY_DEGREE - 1);
    let p2 = rand_poly(MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MAX_POLY_DEGREE - 1);
        assert!(p2.degree() <= MAX_POLY_DEGREE - 1);
    }

    let expected = cyclotomic_mul(&p1, &p2);
    assert!(expected.degree() <= MAX_POLY_DEGREE);
    let res = flat_karatsuba_mul(&p1, &p2);
    assert!(res.degree() <= MAX_POLY_DEGREE);

    assert_eq!(expected, res);
}

/// Test flat Karatsuba cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_both_karatsuba() {
    let p1 = rand_poly(MAX_POLY_DEGREE - 1);
    let p2 = rand_poly(MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MAX_POLY_DEGREE - 1);
        assert!(p2.degree() <= MAX_POLY_DEGREE - 1);
    }

    let expected = karatsuba_mul(&p1, &p2);
    assert!(expected.degree() <= MAX_POLY_DEGREE);
    let res = flat_karatsuba_mul(&p1, &p2);
    assert!(res.degree() <= MAX_POLY_DEGREE);

    assert_eq!(expected, res);
}
