//! Tests for polynomial inverse.

use ark_ff::{One, Zero};
use rand::Rng;

use crate::primitives::poly::{Coeff, Poly};

#[cfg(test)]
use crate::primitives::poly::{inverse, FULL_RES_POLY_DEGREE};

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
    // For small degree and coefficient modulus, the situation may change.
    let out = inverse(&f);
    assert!(out.is_ok());
    assert_eq!(f * out.expect("just checked ok"), Poly::one());
}
