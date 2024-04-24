//! Cyclotomic polynomials using ark-poly.
//! This module file is import-only, code is in submodules:
//! - [`Poly`] is in [`modular_poly`] and its submodules,
//! - [`Coeff`] is in [`fq`] and submodules.

use crate::primitives::poly::fq::rand_coeff;
use ark_ff::{Field, One, Zero};
use ark_poly::polynomial::Polynomial;
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};

pub use fq::Coeff;
pub use modular_poly::{
    modulus::{mod_poly, new_unreduced_poly_modulus_slow, FULL_RES_POLY_DEGREE},
    mul::mul_poly,
    Poly,
};

// Use `mod_poly` outside this module, it is set to the fastest modulus operation.
#[cfg(any(test, feature = "benchmark"))]
pub use modular_poly::modulus::{mod_poly_ark_ref_slow, mod_poly_manual_mut};

// Use `mul_poly` outside this module, it is set to the fastest multiplication operation.
#[cfg(any(test, feature = "benchmark"))]
pub use modular_poly::mul::{
    flat_karatsuba_mul, naive_cyclotomic_mul, poly_split, poly_split_half, rec_karatsuba_mul,
};

pub mod fq;
pub mod modular_poly;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

// Do not add code here.
// Add functions or trait impls to modular_poly/*.rs and inherent method impls to modular_poly.rs.

/// Implementation based on Algorithm 3.3.1 (Page 118) from
/// "A Course in Computational Algebraic Number Theory", Henri Cohen.
/// We don't divide by content of `a` and `b` every time,
/// just in the end of the algorithm.
/// Returns the primitive polynomial which is the inverse of `a` in the
/// cyclotomic ring, if it exists. Otherwise, returns an error.
///
/// When `d` is a constant polynomial and `a` is the polynomial modulus
/// (which reduces to `0`), we have that `b/cont(d)` is the primitive
/// multiplicative inverse of `y`.
pub fn inverse<const MAX_POLY_DEGREE: usize>(
    a: &Poly<MAX_POLY_DEGREE>,
) -> Result<Poly<MAX_POLY_DEGREE>, String> {
    let unreduced_mod_pol = Poly::new_unreduced_poly_modulus_slow();

    let (_x, y, d) = extended_gcd(&unreduced_mod_pol, a);

    // If `d` is a non-zero constant, we can compute the inverse of `d`,
    // and calculate the final primitive inverse.
    if d.is_zero() {
        Err("Can't invert the zero polynomial".to_string())
    } else if d.degree() > 0 {
        Err("Non-invertible polynomial".to_string())
    } else {
        // Reduce to a primitive polynomial.
        let mut inv: Poly<MAX_POLY_DEGREE> = y;
        // Compute the inverse of the content
        let content_inv: Coeff = d[0].inverse().expect("just checked for zero");
        // Divide by `content_inv`
        inv *= content_inv;

        Ok(inv)
    }
}

/// Helps to calculate the equation `cur = prev - q.cur`.
fn update_diophantine<const MAX_POLY_DEGREE: usize>(
    mut prev: Poly<MAX_POLY_DEGREE>,
    cur: Poly<MAX_POLY_DEGREE>,
    q: &Poly<MAX_POLY_DEGREE>,
) -> (Poly<MAX_POLY_DEGREE>, Poly<MAX_POLY_DEGREE>) {
    let mul_res = &cur * q;
    let new_prev = cur;

    prev -= mul_res;
    let new_cur = prev;

    (new_cur, new_prev)
}

/// Returns polynomials `(x, y, d)` such that `a.x + b.y = d`.
pub fn extended_gcd<const MAX_POLY_DEGREE: usize>(
    a: &Poly<MAX_POLY_DEGREE>,
    b: &Poly<MAX_POLY_DEGREE>,
) -> (
    Poly<MAX_POLY_DEGREE>,
    Poly<MAX_POLY_DEGREE>,
    Poly<MAX_POLY_DEGREE>,
) {
    // Invariant a.xi + b.yi = ri

    // init with x0=1, y0=0, r0=a
    let mut x_prev: Poly<MAX_POLY_DEGREE> = Poly::one();
    let mut y_prev = Poly::zero();
    let mut ri_prev = a.clone();
    // next:     x1=0, y1=1, r1=b
    let mut x_cur = Poly::zero();
    let mut y_cur = Poly::one();
    let mut ri_cur = b.clone();

    let mut q: Poly<MAX_POLY_DEGREE>;

    // Sometimes the inputs can be non-canonical.
    ri_cur.truncate_to_canonical_form();

    // loop until ri_cur = 0
    while !(ri_cur.is_zero()) {
        let ri_aux = ri_cur.clone();
        // TODO: q is just a monomial, then we can optimize the next computation
        (q, ri_cur) = ri_prev
            .divide_with_q_and_r(&ri_cur)
            .expect("just checked that the loop divisor is not zero");
        // Sometimes divide_with_q_and_r() might be returning a non-canonical polynomial
        ri_cur.truncate_to_canonical_form();
        ri_prev = ri_aux;

        // x_cur = x_prev - q.x_cur
        (x_cur, x_prev) = update_diophantine(x_prev, x_cur, &q);
        // y_cur = y_prev - q.y_cur
        (y_cur, y_prev) = update_diophantine(y_prev, y_cur, &q);
    }

    (x_prev, y_prev, ri_prev)
}

/// This sampling is similar to what will be necessary for YASHE KeyGen.
/// The purpose is to obtain a polynomial with small random coefficients.
pub fn sample_gaussian<const MAX_POLY_DEGREE: usize>(mut rng: ThreadRng) -> Poly<MAX_POLY_DEGREE> {
    let mut res = Poly::zero();
    // TODO: assert that this is less than the modulus of the coefficient
    for i in 0..MAX_POLY_DEGREE {
        let normal = Normal::new(0.0, 3.0).unwrap();
        let v: f64 = normal.sample(&mut rng);
        res[i] = Coeff::from(v as i64);
    }
    res[0] += Coeff::one();
    res.truncate_to_canonical_form();
    res
}

/// This sampling is similar to what will be necessary for YASHE KeyGen.
/// The purpose is to obtain a polynomial with small random coefficients.
pub fn sample_rand<const MAX_POLY_DEGREE: usize>(mut rng: ThreadRng) -> Poly<MAX_POLY_DEGREE> {
    let mut res = Poly::zero();
    // TODO: assert that this is less than the modulus of the coefficient
    for i in 0..MAX_POLY_DEGREE {
        // TODO: implement Coeff:rand
        let coeff_rand = rand_coeff(&mut rng);
        res[i] = coeff_rand;
    }
    res.truncate_to_canonical_form();
    res
}
