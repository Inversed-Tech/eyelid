//! Cyclotomic polynomial operations using ark-poly.
//!
//! This module contains the base implementations of complex polynomial operations, such as multiplication and reduction.

use ark_ff::Field;
use std::ops::{Add, Sub};

use ark_ff::One;
use ark_ff::Zero;
use ark_poly::polynomial::Polynomial;
use ark_poly::univariate::DenseOrSparsePolynomial;
pub use fq::{Coeff, MAX_POLY_DEGREE};
pub use modular_poly::{
    modulus::{mod_poly, POLY_MODULUS},
    Poly,
};
use std::ops::Mul;

// Use `mod_poly` outside this module, it is set to the fastest modulus operation.
#[cfg(not(any(test, feature = "benchmark")))]
use modular_poly::modulus::{mod_poly_ark_ref, mod_poly_manual_mut};
#[cfg(any(test, feature = "benchmark"))]
pub use modular_poly::modulus::{mod_poly_ark_ref, mod_poly_manual_mut};

pub mod fq;
pub mod modular_poly;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

// TODO: move low-level multiplication code to `modular_poly::mul`

/// The fastest available cyclotomic polynomial multiplication operation (multiply then reduce).
pub use cyclotomic_mul as mul_poly;

/// Minimum degree for recursive Karatsuba calls
pub const MIN_KARATSUBA_REC_DEGREE: usize = 32; // TODO: fine tune

/// Returns a Boolean indicating if the input is equal or not to the additive
/// identity in the polynomial ring
pub fn is_zero_poly(a: Poly) -> bool {
    let mut poly = Poly::zero();
    poly.coeffs = vec![Coeff::zero(); MAX_POLY_DEGREE + 1];
    poly == a
}

/// Returns the multiplicative element of the polynomial ring.
pub fn one_poly(degree: usize) -> Poly {
    assert!(degree <= MAX_POLY_DEGREE);

    let mut poly = Poly::zero();
    poly.coeffs = vec![Coeff::zero(); degree + 1];
    poly.coeffs[0] = Coeff::one();
    poly
}

/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn cyclotomic_mul(a: &Poly, b: &Poly) -> Poly {
    // TODO: change these assertions to debug_assert!() to avoid panics in production code.
    assert!(a.degree() <= MAX_POLY_DEGREE);
    assert!(b.degree() <= MAX_POLY_DEGREE);

    let mut res: Poly = a.naive_mul(b).into();

    // debug_assert_eq!() always needs its arguments, even when the assertion itself is
    // conditionally compiled out using `if cfg!(debug_assertions)`.
    // But when the assertion isn't compiled, the values of the arguments don't matter.
    let dividend = if cfg!(debug_assertions) {
        res.clone()
    } else {
        Poly::zero()
    };

    // Manually ensure the polynomial is reduced and in canonical form,
    // so that we can check the alternate implementation in tests.
    //
    // Use the faster operation between mod_poly_manual*() and mod_poly_ark*() here,
    // and debug_assert_eq!() the other one.
    mod_poly_manual_mut(&mut res);
    debug_assert_eq!(res, mod_poly_ark_ref(&dividend));

    assert!(res.degree() <= MAX_POLY_DEGREE);

    res
}

/// Returns `a * b` followed by reduction mod `XˆN + 1` using recursive Karatsuba method.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn karatsuba_mul(a: &Poly, b: &Poly) -> Poly {
    let mut res;
    let n = a.degree() + 1;

    // if a or b has degree less than min, condition is true
    let cond_a = a.degree() + 1 < MIN_KARATSUBA_REC_DEGREE;
    let cond_b = b.degree() + 1 < MIN_KARATSUBA_REC_DEGREE;
    let rec_cond = cond_a || cond_b;
    if rec_cond {
        // If degree is less than the recursion minimum, just use the naive version
        res = cyclotomic_mul(a, b);
    } else {
        // Otherwise recursively call for al.bl and ar.br
        let (al, ar) = poly_split(a);
        let (bl, br) = poly_split(b);
        let albl = karatsuba_mul(&al, &bl);
        let arbr = karatsuba_mul(&ar, &br);
        let alpar = al.add(ar);
        let blpbr = bl.add(br);
        // Compute y = (al + ar).(bl + br)
        let y = karatsuba_mul(&alpar, &blpbr);
        // Compute res = al.bl + (y - al.bl - ar.br)xˆn/2 + (ar.br)x^n
        res = y.clone();
        res = res.sub(&albl);
        res = res.sub(&arbr);

        // If `a` is reduced, then `xnb2` will never need to be reduced.
        let halfn = n / 2;
        let xnb2 = Poly::xn(halfn);

        res = cyclotomic_mul(&res.clone(), &xnb2);
        res = res.add(albl);
        if n >= MAX_POLY_DEGREE {
            // negate ar.br if n is equal to the max degree (edge case)
            res = res.sub(&arbr);
        } else {
            // Otherwise proceed as usual
            //
            // Even if `a` is reduced, `n` can still be over the maximum degree.
            // But it will only reduce in the initial case, when `a` is the maximum reduced degree.
            // And the reduction is quick, because it is only a single index.
            let mut xn = Poly::xn(n);
            xn.reduce_mod_poly();

            let aux = cyclotomic_mul(&arbr, &xn);
            res = res.add(aux);
        }

        // After manually modifying the leading coefficients, ensure polynomials are in canonical form.
        res.truncate_to_canonical_form();
    };

    res
}

/// Split the polynomial into left and right parts.
pub fn poly_split(a: &Poly) -> (Poly, Poly) {
    // TODO: review performance
    let n = a.degree() + 1;
    let halfn = n / 2;

    let mut al = a.clone();
    let ar = al.coeffs.split_off(halfn);

    // After manually modifying the leading coefficients, ensure polynomials are in canonical form.
    al.truncate_to_canonical_form();
    let ar = Poly::from_coefficients_vec(ar);

    (al, ar)
}

/// Returns the inverse in the cyclotomic ring, if it exists.
/// Otherwise, returns an error.
pub fn inverse(a: &Poly) -> Result<Poly, String> {
    let mut mod_pol = Poly::zero();

    mod_pol[MAX_POLY_DEGREE] = Coeff::one();
    mod_pol[0] = Coeff::one();

    let y = extended_gcd(&mod_pol, a);
    let mul_both = a.clone().mul(y.clone());
    if mul_both.is_one() {
        Ok(y)
    } else {
        Err("No inverse in the ring.".to_owned())
    }
}

/// Helps to calculate the equation `cur = prev - q.cur`.
fn update_diophantine(prev: Poly, cur: Poly, q: Poly) -> (Poly, Poly) {
    let aux = cur.clone();
    let mul_res = cur.mul(&q.clone());
    let new_cur = prev.sub(&mul_res).clone();
    let new_prev = aux.clone();

    (new_cur, new_prev)
}

/// Returns polynomials x, y, d such that a.x + b.y = d.
/// When d=1 we have that x is the multiplicative inverse of a.
pub fn extended_gcd(a: &Poly, b: &Poly) -> Poly {
    // Invariant a.xi + b.yi = ri

    // init with x0=1, y0=0, r0=a
    let mut x_prev = Poly::one();
    let mut y_prev = Poly::zero();
    let mut ri_prev = a.clone();
    // next:     x1=0, y1=1, r1=b
    let mut x_cur = Poly::zero();
    let mut y_cur = Poly::one();
    let ri_cur = b.clone();

    let ri_aux = ri_cur.clone();
    // TODO: q is just a monomial, then we can optimize the next computation
    let (mut q, mut ri_cur) = DenseOrSparsePolynomial::from(ri_prev.clone())
        .divide_with_q_and_r(&ri_cur.into())
        .expect("init divisor is not zero");
    ri_prev = ri_aux;
    // x_cur = x_prev - q.x_cur
    (x_cur, x_prev) = update_diophantine(x_prev, x_cur, q.clone().into());
    // y_cur = y_prev - q.y_cur
    (y_cur, y_prev) = update_diophantine(y_prev, y_cur, q.clone().into());
    // loop until ri_cur = 0
    while !(ri_cur.is_zero()) {
        let ri_aux = ri_cur.clone();
        // TODO: q is just a monomial, then we can optimize the next computation
        (q, ri_cur) = DenseOrSparsePolynomial::from(ri_prev.clone())
            .divide_with_q_and_r(&ri_cur.into())
            .expect("loop divisor is not zero");
        ri_prev = ri_aux.into();
        // x_cur = x_prev - q.x_cur
        (x_cur, x_prev) = update_diophantine(x_prev, x_cur, q.clone().into());
        // y_cur = y_prev - q.y_cur
        (y_cur, y_prev) = update_diophantine(y_prev, y_cur, q.clone().into());
    }
    // compute ri_prev inverse to calculate the final result
    let divisor = ri_prev.clone();
    // FIXME: if b is not invertible mod a the assert is not going to pass.
    // Since the test is probabilistic, we have a small chance of failure.
    // It fails after a small number of executions.
    // Sometimes it fails with a different error, meaning there is another
    // problem happening.
    // It occasionally fails when we do the following:
    // export RUSTFLAGS="-D warnings --cfg tiny_poly"
    // cargo test --all-targets -- --nocapture inv
    debug_assert!(divisor.degree() == 0);
    let divisor_inv = divisor[0].inverse().unwrap();
    // y_cur / ri_prev
    let mut final_result = y_prev.clone();
    for i in 0..=y_prev.degree() {
        final_result[i] = final_result[i].mul(divisor_inv);
    }
    final_result
}
