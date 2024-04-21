//! Tests for basic polynomial operations.

use ark_ff::{One, Zero};
use ark_poly::{univariate::DenseOrSparsePolynomial, Polynomial};

use crate::primitives::poly::{
    flat_karatsuba_mul, naive_cyclotomic_mul, new_unreduced_poly_modulus_slow, rec_karatsuba_mul,
    test::gen::rand_poly, Coeff, Poly, FULL_RES_POLY_DEGREE,
};

/// Test cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_cyclotomic_mul_rand_xnm1() {
    check_cyclotomic_mul_rand_xnm1::<FULL_RES_POLY_DEGREE, _>(naive_cyclotomic_mul);
    check_cyclotomic_mul_rand_xnm1::<FULL_RES_POLY_DEGREE, _>(rec_karatsuba_mul);
    check_cyclotomic_mul_rand_xnm1::<FULL_RES_POLY_DEGREE, _>(flat_karatsuba_mul);
}

/// Check `mul_fn` correctly implements cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
fn check_cyclotomic_mul_rand_xnm1<const MAX_POLY_DEGREE: usize, F>(mul_fn: F)
where
    F: Fn(&Poly<MAX_POLY_DEGREE>, &Poly<MAX_POLY_DEGREE>) -> Poly<MAX_POLY_DEGREE>,
{
    let p1: Poly<MAX_POLY_DEGREE> = rand_poly(MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MAX_POLY_DEGREE - 1);
    }

    // Xˆ{N-1}, multiplying by it will rotate by N-1 and negate (except the first).
    let xnm1 = Poly::xn(MAX_POLY_DEGREE - 1);

    assert_eq!(xnm1.degree(), MAX_POLY_DEGREE - 1);

    let res = mul_fn(&p1, &xnm1);
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
    check_cyclotomic_mul_max_degree::<FULL_RES_POLY_DEGREE, _>(naive_cyclotomic_mul);
    check_cyclotomic_mul_max_degree::<FULL_RES_POLY_DEGREE, _>(rec_karatsuba_mul);
    check_cyclotomic_mul_max_degree::<FULL_RES_POLY_DEGREE, _>(flat_karatsuba_mul);
}

/// Check `mul_fn` correctly implements cyclotomic multiplication that results in `X^[MAX_POLY_DEGREE]`.
fn check_cyclotomic_mul_max_degree<const MAX_POLY_DEGREE: usize, F>(mul_fn: F)
where
    F: Fn(&Poly<MAX_POLY_DEGREE>, &Poly<MAX_POLY_DEGREE>) -> Poly<MAX_POLY_DEGREE>,
{
    // X^MAX_POLY_DEGREE
    //
    // Since the degree is equal to MAX_POLY_DEGREE, this is not reduced.
    // But it is in canonical form, because the leading coefficient is non-zero.
    let mut x_max: Poly<MAX_POLY_DEGREE> = Poly::zero();
    x_max[MAX_POLY_DEGREE] = Coeff::one();

    // Manually calculate the reduced representation of X^N as the constant `MODULUS - 1`.
    let x_max = DenseOrSparsePolynomial::from(x_max);
    let (q, x_max) = x_max
        .divide_with_q_and_r(&new_unreduced_poly_modulus_slow::<MAX_POLY_DEGREE>().into())
        .expect("is divisible by X^MAX_POLY_DEGREE");
    let q: Poly<MAX_POLY_DEGREE> = q.into();
    let x_max: Poly<MAX_POLY_DEGREE> = x_max.into();

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

        if i == 0 || i == FULL_RES_POLY_DEGREE {
            assert_eq!(p1.degree(), 0);
            assert_eq!(p2.degree(), 0);
        } else {
            assert_eq!(p1.degree() + p2.degree(), MAX_POLY_DEGREE);
        }

        let res = mul_fn(&p1, &p2);

        // Make sure it's X^N
        assert_eq!(res, x_max, "x^{i} * x^{}", MAX_POLY_DEGREE - i);
    }
}

/// Test recursive karatsuba, flat karatsuba, and naive cyclotomic multiplication of two random polynomials all produce the same result.
#[test]
fn test_karatsuba_mul_rand_consistent() {
    let p1: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE - 1);
    let p2: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= FULL_RES_POLY_DEGREE - 1);
        assert!(p2.degree() <= FULL_RES_POLY_DEGREE - 1);
    }

    let expected = naive_cyclotomic_mul(&p1, &p2);
    assert!(expected.degree() <= FULL_RES_POLY_DEGREE);

    let rec_res = rec_karatsuba_mul(&p1, &p2);
    assert!(rec_res.degree() <= FULL_RES_POLY_DEGREE);

    let flat_res = flat_karatsuba_mul(&p1, &p2);
    assert!(flat_res.degree() <= FULL_RES_POLY_DEGREE);

    assert_eq!(expected, rec_res);
    assert_eq!(expected, flat_res);
}
