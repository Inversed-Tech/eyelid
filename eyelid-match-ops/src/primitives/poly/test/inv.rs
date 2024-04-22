//! Tests for polynomial inverse.

use ark_ff::{One, Zero};
use rand::Rng;

use crate::primitives::poly::{Coeff, Poly};

#[cfg(test)]
use crate::primitives::poly::{extended_gcd, inverse, FULL_RES_POLY_DEGREE};
#[cfg(test)]
use ark_poly::Polynomial;

/// This sampling is similar to what will be necessary for YASHE KeyGen
/// TODO: generate Gaussian distribution instead of "uniform"
pub fn sample<const MAX_POLY_DEGREE: usize>() -> Poly<MAX_POLY_DEGREE> {
    let mut rng = rand::thread_rng();
    let mut res = Poly::zero();
    let max_coeff = 8;
    let t = 2;
    for i in 0..MAX_POLY_DEGREE {
        let coeff_rand = rng.gen_range(1..max_coeff);
        res[i] = Coeff::from(t * coeff_rand);
    }
    res[0] += Coeff::one();
    res
}

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
