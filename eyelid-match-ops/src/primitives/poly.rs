//! Cyclotomic polynomial operations using ark-poly.
//!
//! This module contains the base implementations of complex polynomial operations, such as multiplication and reduction.

use std::ops::{Add, Sub};

use ark_ff::Zero;
use ark_poly::polynomial::Polynomial;

pub use fq::Coeff;
pub use modular_poly::{
    modulus::{mod_poly, FULL_RES_POLY_DEGREE},
    Poly,
};

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

/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// All polynomials have maximum degree `MAX_POLY_DEGREE`.
pub fn cyclotomic_mul<const MAX_POLY_DEGREE: usize>(
    a: &Poly<MAX_POLY_DEGREE>,
    b: &Poly<MAX_POLY_DEGREE>,
) -> Poly<MAX_POLY_DEGREE> {
    // TODO: change these assertions to debug_assert!() to avoid panics in production code.
    assert!(a.degree() <= MAX_POLY_DEGREE);
    assert!(b.degree() <= MAX_POLY_DEGREE);

    let mut res: Poly<MAX_POLY_DEGREE> = a.naive_mul(b).into();

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
/// All polynomials have maximum degree `MAX_POLY_DEGREE`.
pub fn karatsuba_mul<const MAX_POLY_DEGREE: usize>(
    a: &Poly<MAX_POLY_DEGREE>,
    b: &Poly<MAX_POLY_DEGREE>,
) -> Poly<MAX_POLY_DEGREE> {
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
/// All polynomials have maximum degree `MAX_POLY_DEGREE`. The modulus remains the same even after
/// the split.
pub fn poly_split<const MAX_POLY_DEGREE: usize>(
    a: &Poly<MAX_POLY_DEGREE>,
) -> (Poly<MAX_POLY_DEGREE>, Poly<MAX_POLY_DEGREE>) {
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
