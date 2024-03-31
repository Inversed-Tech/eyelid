//! Tests for polynomial inverse.

use super::gen::rand_poly;

use super::super::*;

#[test]
fn test_extended_gcd() {
    let mut p1 = rand_poly(MAX_POLY_DEGREE - 1);

    let mut xnm1 = zero_poly(MAX_POLY_DEGREE - 1);
    xnm1.coeffs[MAX_POLY_DEGREE - 1] = Coeff::one();

    let res = extended_gcd(&xnm1, p1);

    dbg!(res);
}
