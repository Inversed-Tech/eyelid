//! Tests for basic polynomial operations.

use super::gen::rand_poly;

use super::super::*;

/// Test cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_cyclotomic_mul_rand() {
    let p1 = rand_poly(MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MAX_POLY_DEGREE - 1);
    }

    // Xˆ{N-1}, multiplying by it will rotate by N-1 and negate (except the first).
    let mut xnm1 = zero_poly(MAX_POLY_DEGREE - 1);
    xnm1.coeffs[MAX_POLY_DEGREE - 1] = Coeff::one();

    assert_eq!(xnm1.degree(), MAX_POLY_DEGREE - 1);

    let res = cyclotomic_mul(&p1, &xnm1);
    assert!(res.degree() <= MAX_POLY_DEGREE);

    for i in 0..MAX_POLY_DEGREE - 1 {
        // Negative numbers are automatically converted to canonical
        // representation in the interval [0, MODULUS)
        assert_eq!(res[i], -p1[i + 1]);
    }
    assert_eq!(res[MAX_POLY_DEGREE - 1], p1[0]);

    // Zero coefficients aren't stored.
    if res.degree() >= MAX_POLY_DEGREE {
        for i in (MAX_POLY_DEGREE)..=res.degree() {
            assert_eq!(res[i], Coeff::zero());
        }
    }
}

/// Test cyclotomic multiplication that results in `X^[MAX_POLY_DEGREE]`.
#[test]
fn test_cyclotomic_mul_max_degree() {
    use ark_poly::DenseUVPolynomial;

    // X^MAX_POLY_DEGREE
    let mut x_max = zero_poly(MAX_POLY_DEGREE);
    x_max[MAX_POLY_DEGREE] = Coeff::one();

    // There is a shorter representation of X^N as the constant `MODULUS - 1`.
    let x_max = DenseOrSparsePolynomial::from(x_max);
    let (q, x_max) = x_max
        .divide_with_q_and_r(&*POLY_MODULUS)
        .expect("is divisible by X^MAX_POLY_DEGREE");

    assert_eq!(q, Poly::from_coefficients_vec(vec![Coeff::one()]));
    assert_eq!(
        x_max,
        // TODO: should this be a constant?
        Poly::from_coefficients_vec(vec![Coeff::zero() - Coeff::one()]),
    );

    for i in 0..=MAX_POLY_DEGREE {
        // X^i * X^{MAX_POLY_DEGREE - i} = X^MAX_POLY_DEGREE
        let mut p1 = zero_poly(i);
        p1[i] = Coeff::one();

        let mut p2 = zero_poly(MAX_POLY_DEGREE - i);
        p2[MAX_POLY_DEGREE - i] = Coeff::one();

        assert_eq!(p1.degree() + p2.degree(), MAX_POLY_DEGREE);

        let res = cyclotomic_mul(&p1, &p2);

        // Make sure it's X^N
        assert_eq!(res, x_max);
    }
}

/// Test cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
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
