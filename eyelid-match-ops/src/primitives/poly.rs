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

/// Returns the inverse in the cyclotomic ring, if it exists.
/// Otherwise, returns an error.
pub fn inverse<const MAX_POLY_DEGREE: usize>(
    a: &Poly<MAX_POLY_DEGREE>,
) -> Result<Poly<MAX_POLY_DEGREE>, String> {
    let unreduced_mod_pol = Poly::new_unreduced_poly_modulus_slow();

    extended_gcd(&unreduced_mod_pol, a)
}

/// Helps to calculate the equation `cur = prev - q.cur`.
fn update_diophantine<const MAX_POLY_DEGREE: usize>(
    prev: Poly<MAX_POLY_DEGREE>,
    cur: Poly<MAX_POLY_DEGREE>,
    q: &Poly<MAX_POLY_DEGREE>,
) -> (Poly<MAX_POLY_DEGREE>, Poly<MAX_POLY_DEGREE>) {
    let new_prev = cur.clone();
    let mul_res = cur * q;
    let new_cur = prev - mul_res;

    (new_cur, new_prev)
}

/// Returns the multiplicative inverse of `b`.
///
/// Calculates polynomials such that `a.x + b.y = d`.
/// When `d=0` and `a` is the polynomial modulus, we have that `b` is the multiplicative inverse of `y`.
/// Otherwise, returns an error.
pub fn extended_gcd<const MAX_POLY_DEGREE: usize>(
    a: &Poly<MAX_POLY_DEGREE>,
    b: &Poly<MAX_POLY_DEGREE>,
) -> Result<Poly<MAX_POLY_DEGREE>, String> {
    // Invariant a.xi + b.yi = ri

    // init with x0=1, y0=0, r0=a
    let mut x_prev: Poly<MAX_POLY_DEGREE> = Poly::one();
    let mut y_prev = Poly::zero();
    let mut ri_prev = a.clone();
    // next:     x1=0, y1=1, r1=b
    let mut x_cur = Poly::zero();
    let mut y_cur = Poly::one();
    let mut ri_cur = b.clone();
    let mut q;

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
    // compute ri_prev inverse to calculate the final result
    if ri_prev.degree() == 0 {
        let divisor_inv = ri_prev[0].inverse().unwrap();
        // y_cur / ri_prev
        let mut final_result = y_prev.clone();
        for i in 0..=y_prev.degree() {
            final_result[i] *= divisor_inv;
        }
        Ok(final_result)
    } else {
        Err("Can't invert b, invalid divisor".to_owned())
    }
}
