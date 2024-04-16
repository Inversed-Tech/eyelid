//! Tests for polynomial inverse.

use super::super::*;
use rand::Rng;

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
    loop {
        let f = sample();
        let out = inverse(&f);
        if !out.is_err() {
            out.unwrap();
            break;
        }
    }
}
