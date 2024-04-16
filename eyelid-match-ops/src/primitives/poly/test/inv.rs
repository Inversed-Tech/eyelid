//! Tests for polynomial inverse.

use super::gen::rand_poly;

use super::super::*;
use rand::Rng;

/*#[test]
fn test_extended_gcd() {
    let mut p1 = rand_poly(MAX_POLY_DEGREE - 1);

    let mut xnm1 = zero_poly(MAX_POLY_DEGREE - 1);
    xnm1.coeffs[MAX_POLY_DEGREE - 1] = Coeff::one();

    let res = extended_gcd(&xnm1, p1);

    dbg!(res);
}*/

fn sample() -> Poly {
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
    let d = Poly::zero();

    let mut res;
    loop {
        let f = sample();
        let out = inverse(&f);
        if !out.is_err() {
            res = out.unwrap();
            break;
        }
    };

}
