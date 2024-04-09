//! Cyclotomic polynomial operations using ark-poly.
//!
//! This module contains the base implementations of complex polynomial operations, such as multiplication and reduction.

use std::ops::{Add, Sub};

use ark_ff::One;
use ark_poly::polynomial::Polynomial;

pub use fq::{Coeff, MAX_POLY_DEGREE};
pub use modular_poly::{mod_poly, zero_poly, Poly, POLY_MODULUS};

// Use `mod_poly` outside this module, it is set to the fastest modulus operation.
#[cfg(not(any(test, feature = "benchmark")))]
use modular_poly::{mod_poly_ark_ref, mod_poly_manual_mut};
#[cfg(any(test, feature = "benchmark"))]
pub use modular_poly::{mod_poly_ark_ref, mod_poly_manual_mut};

pub mod fq;
pub mod modular_poly;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

/// Minimum degree for recursive Karatsuba calls
pub const MIN_KARATSUBA_REC_DEGREE: usize = 32; // TODO: fine tune

/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn cyclotomic_mul(a: &Poly, b: &Poly) -> Poly {
    // TODO: change these assertions to debug_assert!() to avoid panics in production code.
    assert!(a.degree() <= MAX_POLY_DEGREE);
    assert!(b.degree() <= MAX_POLY_DEGREE);

    let mut res: Poly = a.naive_mul(b).into();
    #[cfg(debug_assertions)]
    let dividend = res.clone();

    // Use the fastest benchmark between mod_poly_manual*() and mod_poly_ark*() here,
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

        let halfn = n / 2;
        let mut xnb2 = zero_poly(halfn);
        xnb2.coeffs[halfn] = Coeff::one();

        res = cyclotomic_mul(&res.clone(), &xnb2);
        res = res.add(albl);
        if n >= MAX_POLY_DEGREE {
            // negate ar.br if n is equal to the max degree (edge case)
            res = res.sub(&arbr);
        } else {
            // Otherwise proceed as usual
            let mut xn = zero_poly(n);
            xn.coeffs[n] = Coeff::one();
            let aux = cyclotomic_mul(&arbr, &xn);
            res = res.add(aux);
        }
    };

    // After manually modifying the leading coefficients, ensure polynomials are in canonical form.
    res.truncate_to_canonical_form();
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
