//! Cyclotomic polynomials using ark-poly.
//! This module file is import-only, code is in submodules:
//! - [`Poly`] is in [`modular_poly`] and its submodules,
//! - [`Coeff`] is in [`fq`] and submodules.

use ark_ff::{Field, One, Zero};
use ark_poly::polynomial::Polynomial;

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

/// Returns the monic inverse of `a` in the cyclotomic ring, if it exists.
/// Otherwise, returns an error.
pub fn inverse<const MAX_POLY_DEGREE: usize>(
    a: &Poly<MAX_POLY_DEGREE>,
) -> Result<Poly<MAX_POLY_DEGREE>, String> {
    let unreduced_mod_pol = Poly::new_unreduced_poly_modulus_slow();

    let (_x, y, d) = extended_gcd(&unreduced_mod_pol, a);

    // If `d` is a non-zero constant, we can compute the inverse of `d`,
    // and calculate the final inverse.
    if d.is_zero() {
        Err("Can't invert the zero polynomial".to_string())
    } else if d.degree() > 0 {
        Err("Non-invertible polynomial".to_string())
    } else {
        // Reduce to a monic polynomial.
        let mut inv: Poly<MAX_POLY_DEGREE> = y;
        let divisor_inv: Coeff = d[0].inverse().expect("just checked for zero");

        inv *= divisor_inv;

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
///
/// Calculates polynomials such that `a.x + b.y = d`.
/// When `d=1` and `a` is the polynomial modulus (which reduces to `0`),
/// we have that `b` is the multiplicative inverse of `y`.
/// Otherwise, returns an error.
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

    // loop until ri_cur = 0
    while !(ri_cur.is_zero()) {
        let ri_aux = ri_cur.clone();
        // TODO: q is just a monomial, then we can optimize the next computation
        (q, ri_cur) = ri_prev
            .divide_with_q_and_r(&ri_cur)
            .expect("just checked that the loop divisor is not zero");
        ri_prev = ri_aux;

        // x_cur = x_prev - q.x_cur
        (x_cur, x_prev) = update_diophantine(x_prev, x_cur, &q);
        // y_cur = y_prev - q.y_cur
        (y_cur, y_prev) = update_diophantine(y_prev, y_cur, &q);
    }

    (x_prev, y_prev, ri_prev)
}
