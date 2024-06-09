//! Tests for polynomial multiplication.

use std::any::type_name;

use ark_ff::{One, Zero};
use ark_poly::Polynomial;

use crate::{
    primitives::poly::{
        flat_karatsuba_mul, naive_cyclotomic_mul, new_unreduced_poly_modulus_slow,
        rec_karatsuba_mul, test::gen::rand_poly, Poly, PolyConf,
    },
    MiddleRes, TestRes,
};

/// Test cyclotomic multiplication of a random polynomial by `X^{[C::MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_cyclotomic_mul_rand_xnm1() {
    check_cyclotomic_mul_rand_xnm1::<TestRes, _>(naive_cyclotomic_mul);
    check_cyclotomic_mul_rand_xnm1::<TestRes, _>(rec_karatsuba_mul);
    check_cyclotomic_mul_rand_xnm1::<TestRes, _>(flat_karatsuba_mul);

    check_cyclotomic_mul_rand_xnm1::<MiddleRes, _>(naive_cyclotomic_mul);
    check_cyclotomic_mul_rand_xnm1::<MiddleRes, _>(rec_karatsuba_mul);
    check_cyclotomic_mul_rand_xnm1::<MiddleRes, _>(flat_karatsuba_mul);
}

/// Check `mul_fn` correctly implements cyclotomic multiplication of a random polynomial by `X^{[C::MAX_POLY_DEGREE] - 1}`.
fn check_cyclotomic_mul_rand_xnm1<C: PolyConf, F>(mul_fn: F)
where
    F: Fn(&Poly<C>, &Poly<C>) -> Poly<C>,
{
    let p1: Poly<C> = rand_poly(C::MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(
            p1.degree() <= C::MAX_POLY_DEGREE - 1,
            "{}",
            type_name::<C>()
        );
    }

    // Xˆ{N-1}, multiplying by it will rotate by N-1 and negate (except the first).
    let xnm1 = Poly::xn(C::MAX_POLY_DEGREE - 1);

    assert_eq!(
        xnm1.degree(),
        C::MAX_POLY_DEGREE - 1,
        "{}",
        type_name::<C>()
    );

    let res = mul_fn(&p1, &xnm1);
    assert!(res.degree() <= C::MAX_POLY_DEGREE, "{}", type_name::<C>());

    for i in 0..C::MAX_POLY_DEGREE - 1 {
        // Negative numbers are automatically converted to canonical
        // representation in the interval [0, MODULUS)
        assert_eq!(res[i], -p1[i + 1], "{}", type_name::<C>());
    }
    assert_eq!(res[C::MAX_POLY_DEGREE - 1], p1[0], "{}", type_name::<C>());

    // Zero leading coefficients aren't stored.
    // `degree()` panics if the leading coefficient is zero anyway.
    assert!(res.degree() < C::MAX_POLY_DEGREE, "{}", type_name::<C>());
}

/// Test cyclotomic multiplication that results in `X^[C::MAX_POLY_DEGREE]`.
#[test]
fn test_cyclotomic_mul_max_degree() {
    check_cyclotomic_mul_max_degree::<TestRes, _>(naive_cyclotomic_mul);
    check_cyclotomic_mul_max_degree::<TestRes, _>(rec_karatsuba_mul);
    check_cyclotomic_mul_max_degree::<TestRes, _>(flat_karatsuba_mul);

    check_cyclotomic_mul_max_degree::<MiddleRes, _>(naive_cyclotomic_mul);
    check_cyclotomic_mul_max_degree::<MiddleRes, _>(rec_karatsuba_mul);
    check_cyclotomic_mul_max_degree::<MiddleRes, _>(flat_karatsuba_mul);
}

/// Check `mul_fn` correctly implements cyclotomic multiplication that results in `X^[C::MAX_POLY_DEGREE]`.
fn check_cyclotomic_mul_max_degree<C: PolyConf, F>(mul_fn: F)
where
    F: Fn(&Poly<C>, &Poly<C>) -> Poly<C>,
{
    // X^C::MAX_POLY_DEGREE
    //
    // Create a polynomial with degree equal to C::MAX_POLY_DEGREE.
    // We can't use the standard methods, because we want an un-reduced polynomial.
    // But it is in canonical form, because the leading coefficient is non-zero.
    let mut x_max: Poly<C> = Poly::zero();
    x_max[C::MAX_POLY_DEGREE] = C::Coeff::one();

    // Manually calculate the reduced representation of X^N as the constant `MODULUS - 1`.
    let (q, x_max) = x_max
        .divide_with_q_and_r(&new_unreduced_poly_modulus_slow::<C>())
        .unwrap_or_else(|| panic!("is divisible by X^{}::MAX_POLY_DEGREE", type_name::<C>()));

    assert_eq!(
        q,
        Poly::from_coefficients_vec(vec![C::Coeff::one()]),
        "{}",
        type_name::<C>()
    );
    assert_eq!(
        x_max,
        // TODO: should `MODULUS - 1` be a constant?
        Poly::from_coefficients_vec(vec![C::Coeff::zero() - C::Coeff::one()]),
        "{}",
        type_name::<C>()
    );

    for i in 0..=C::MAX_POLY_DEGREE {
        // This test is slow, so skip most values.
        if i % 101 != 0
            && ![
                0,
                1,
                C::MAX_POLY_DEGREE / 2 - 1,
                C::MAX_POLY_DEGREE / 2,
                C::MAX_POLY_DEGREE / 2 + 1,
                C::MAX_POLY_DEGREE - 1,
                C::MAX_POLY_DEGREE,
            ]
            .contains(&i)
        {
            continue;
        }

        // X^i * X^{C::MAX_POLY_DEGREE - i} = X^C::MAX_POLY_DEGREE

        // `p1` and `p2` are automatically reduced if needed.
        let p1 = Poly::xn(i);
        let p2 = Poly::xn(C::MAX_POLY_DEGREE - i);

        if i == 0 || i == C::MAX_POLY_DEGREE {
            assert_eq!(
                p1.degree(),
                0,
                "{}: p1: X^{i} degree {}: degree is inconsistent",
                type_name::<C>(),
                p1.degree(),
            );
            assert_eq!(
                p2.degree(),
                0,
                "{}: p2: x^{} degree {}: degree is inconsistent",
                type_name::<C>(),
                C::MAX_POLY_DEGREE - i,
                p2.degree(),
            );
        } else {
            assert_eq!(
                p1.degree() + p2.degree(),
                C::MAX_POLY_DEGREE,
                "{}: p1: X^{i} degree {} * p2: x^{} degree {}: degrees are inconsistent",
                type_name::<C>(),
                p1.degree(),
                C::MAX_POLY_DEGREE - i,
                p2.degree(),
            );
        }

        let res = mul_fn(&p1, &p2);

        // Make sure it's X^N
        assert_eq!(
            res,
            x_max,
            "{}: x^{i} * x^{}",
            type_name::<C>(),
            C::MAX_POLY_DEGREE - i
        );
    }
}

/// Test recursive karatsuba, flat karatsuba, and naive cyclotomic multiplication of two random polynomials all produce the same result.
#[test]
fn test_karatsuba_mul_rand_consistent() {
    // TestRes
    let p1: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE - 1);
    let p2: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= TestRes::MAX_POLY_DEGREE - 1);
        assert!(p2.degree() <= TestRes::MAX_POLY_DEGREE - 1);
    }

    let expected = naive_cyclotomic_mul(&p1, &p2);
    assert!(expected.degree() <= TestRes::MAX_POLY_DEGREE);

    let rec_res = rec_karatsuba_mul(&p1, &p2);
    assert!(rec_res.degree() <= TestRes::MAX_POLY_DEGREE);

    let flat_res = flat_karatsuba_mul(&p1, &p2);
    assert!(flat_res.degree() <= TestRes::MAX_POLY_DEGREE);

    assert_eq!(expected, rec_res);
    assert_eq!(expected, flat_res);

    // MiddleRes
    let p1: Poly<MiddleRes> = rand_poly(TestRes::MAX_POLY_DEGREE - 1);
    let p2: Poly<MiddleRes> = rand_poly(TestRes::MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MiddleRes::MAX_POLY_DEGREE - 1);
        assert!(p2.degree() <= MiddleRes::MAX_POLY_DEGREE - 1);
    }

    let expected = naive_cyclotomic_mul(&p1, &p2);
    assert!(expected.degree() <= MiddleRes::MAX_POLY_DEGREE);

    let rec_res = rec_karatsuba_mul(&p1, &p2);
    assert!(rec_res.degree() <= MiddleRes::MAX_POLY_DEGREE);

    let flat_res = flat_karatsuba_mul(&p1, &p2);
    assert!(flat_res.degree() <= MiddleRes::MAX_POLY_DEGREE);

    assert_eq!(expected, rec_res);
    assert_eq!(expected, flat_res);
}
