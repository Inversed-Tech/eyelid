//! Cyclotomic polynomial operations using ark-poly.
//!
//! This module contains the base implementations of complex polynomial operations, such as multiplication and reduction.

use std::ops::{Add, Sub};

use ark_ff::Zero;
use ark_poly::polynomial::Polynomial;

pub use fq::{Coeff, MAX_POLY_DEGREE};
pub use modular_poly::{
    modulus::{mod_poly, POLY_MODULUS},
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
pub const MIN_KARATSUBA_REC_DEGREE: usize = 8; // TODO: fine tune

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
    let n = a.degree() + 1; // invariant: n is a power of 2
    debug_assert!(n.count_ones() == 1); // checking the invariant

    // if a or b has degree less than min, condition is true
    let cond_a = a.degree() <= MIN_KARATSUBA_REC_DEGREE;
    let cond_b = b.degree() <= MIN_KARATSUBA_REC_DEGREE;
    let rec_cond = cond_a || cond_b;
    if rec_cond {
        // If degree is less than the recursion minimum, just use the naive version
        res = a.naive_mul(b);
    } else {
        // Otherwise recursively call for al.bl and ar.br
        let (al, ar) = poly_split_half(a);
        let (bl, br) = poly_split_half(b);
        let albl = karatsuba_mul(&al, &bl);
        let arbr = karatsuba_mul(&ar, &br);
        let alpar = al.add(ar);
        let blpbr = bl.add(br);
        // Compute y = (al + ar).(bl + br)
        let y = karatsuba_mul(&alpar, &blpbr);
        // Compute res = al.bl + (y - al.bl - ar.br)xˆn/2 + (ar.br)x^n
        res = y.sub(&albl);
        res = res.sub(&arbr);

        // `res` will be reduced if needed, but that should only happen once in the first loop.
        let halfn = n / 2;
        res.mul_xn(halfn);
        res = res.add(albl);
        if n >= MAX_POLY_DEGREE {
            // negate ar.br if n is equal to the max degree (edge case)
            res = res.sub(&arbr);
        } else {
            // Otherwise proceed as usual
            arbr. mul_xn(n);

            res = res.add(arbr);
        }

        // After manually modifying the leading coefficients, ensure polynomials are in canonical form.
        res.truncate_to_canonical_form();
    };
  
    res.reduce_mod_poly();
    res
}

/// Flat (without recursion) implementation of Karatsuba.
/// This implementation can be parallelized since for each layer
/// we have that chunks are independent of each other.
pub fn flat_karatsuba_mul(a: &Poly, b: &Poly) -> Poly {
    let n = a.degree() + 1;
    let recursion_height = usize::ilog2(n);

    let mut first_layer_number = 3; // TODO: fine tune
    let mut chunk_size = 2usize.pow(first_layer_number - 1);
    let first_layer_length = MAX_POLY_DEGREE / chunk_size;
    let mut polys_current_layer: Vec<Poly> = vec![];
    let mut polys_next_layer: Vec<Poly> = vec![];
    let a_chunks = poly_split(a, chunk_size);
    let b_chunks = poly_split(b, chunk_size);

    // Take 2 at each step
    for i in 0..first_layer_length / 2 {
        // al, ar
        let al = &a_chunks[2 * i];
        let ar = &a_chunks[2 * i + 1];
        // bl, br
        let bl = &b_chunks[2 * i];
        let br = &b_chunks[2 * i + 1];

        let albl = al.naive_mul(&bl);
        let arbr = ar.naive_mul(&br);
        let alpar = al.add(ar);
        let blpbr = bl.add(br);
        // Compute y = (al + ar).(bl + br)
        let mut res = alpar.naive_mul(&blpbr);

        // Compute res = al.bl + (y - al.bl - ar.br)xˆ1 + (ar.br)x^2
        res = res.sub(&albl);
        res = res.sub(&arbr);
        res.mul_xn(chunk_size);
        res = res.add(albl);

        // along the process part:
        let mut xip1 = zero_poly(2 * chunk_size);
        xip1.coeffs[2 * chunk_size] = Fq79::one();
        // TODO: use specific function for this kind of shift, as described above
        let aux = arbr.naive_mul(&xip1);
        res = res.add(aux);

        polys_current_layer.push(res);
    }
    chunk_size *= 2;

    while first_layer_number < recursion_height {
        let a_chunks = poly_split(a, chunk_size);
        let b_chunks = poly_split(b, chunk_size);
        let layer_length = polys_current_layer.len();
        // Take 2
        debug_assert!(a_chunks.len() == MAX_POLY_DEGREE / chunk_size);
        debug_assert!(a_chunks.len() == b_chunks.len());
        debug_assert!(a_chunks.len() == polys_current_layer.len());
        for j in 0..layer_length / 2 {
            // Take two polynomials each round

            // al, ar
            let al = &a_chunks[2 * j];
            let ar = &a_chunks[2 * j + 1];
            // bl, br
            let bl = &b_chunks[2 * j];
            let br = &b_chunks[2 * j + 1];

            // NOT NEEDED, SINCE IT COMES FROM PREVIOUS LAYER
            //let albl = al.naive_mul(&bl);
            let albl = &polys_current_layer[2 * j];
            //let arbr = ar.naive_mul(&br);
            let arbr = &polys_current_layer[2 * j + 1];
            let alpar = al.add(ar);
            let blpbr = bl.add(br);
            // Compute y = (al + ar).(bl + br)
            let mut res = alpar.naive_mul(&blpbr);

            // Compute res = al.bl + (y - al.bl - ar.br)xˆn/2 + (ar.br)x^n
            res = res.sub(albl);
            res = res.sub(arbr);
            let half_chunk_size = chunk_size;
            res.mul_xn(half_chunk_size);
            res = albl.add(&res);

            arbr.mul_xn(2 * chunk_size);
            res = res.add(arbr);

            polys_next_layer.push(res);
        }
        polys_current_layer = polys_next_layer;
        polys_next_layer = vec![];
        first_layer_number += 1;
        chunk_size *= 2;
    }

    debug_assert!(polys_current_layer.len() == 1);
    polys_current_layer[0].reduce_mod_poly();
    polys_current_layer[0]
}

/// Split the polynomial into left and right parts.
pub fn poly_split(a: &Poly, k: usize) -> Vec<Poly> {
    // TODO: review performance
    // TODO: k must be a power of 2, check it
    a.coeffs.chunks(k).map(Poly::from_coefficients_slice).collect()
}

/// Split the polynomial into left and right parts.
pub fn poly_split_half(a: &Poly) -> (Poly, Poly) {
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
