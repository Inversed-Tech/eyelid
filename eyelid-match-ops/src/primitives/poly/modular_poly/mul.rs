//! Efficient polynomial multiplication.

use std::ops::MulAssign;

use ark_ff::Zero;
use ark_poly::polynomial::Polynomial;
use static_assertions::const_assert_eq;

use crate::primitives::poly::{
    mod_poly,
    modular_poly::modulus::{mod_poly_ark_ref_slow, mod_poly_manual_mut},
    Poly, PolyConf,
};

// Simple multiplication by a field element.

impl<C: PolyConf> MulAssign<C::Coeff> for Poly<C> {
    fn mul_assign(&mut self, rhs: C::Coeff) {
        for coeff in &mut self.0.coeffs {
            *coeff *= rhs;
        }
        self.truncate_to_canonical_form();
    }
}

impl<C: PolyConf> MulAssign<C::Coeff> for &mut Poly<C> {
    fn mul_assign(&mut self, rhs: C::Coeff) {
        for coeff in &mut self.0.coeffs {
            *coeff *= rhs;
        }
        self.truncate_to_canonical_form();
    }
}

/// The fastest available cyclotomic polynomial multiplication operation (multiply then reduce).
pub use rec_karatsuba_mul as mul_poly;

/// Minimum degree for recursive Karatsuba calls.
// TODO: fine tune this constant
#[cfg(not(tiny_poly))]
pub const REC_KARATSUBA_MIN_DEGREE: usize = 8;

/// Tiny test polynomial minimum degree for recursive Karatsuba calls.
#[cfg(tiny_poly)]
pub const REC_KARATSUBA_MIN_DEGREE: usize = 2;

/// Initial layer parameter for the flat Karatsuba loop.
/// The initial layer has polynomials with `2ˆ{FLAT_KARATSUBA_FIRST_LAYER - 1}` coefficients.
//
// TODO: fine tune this constant
#[cfg(not(tiny_poly))]
#[cfg(any(test, feature = "benchmark"))]
pub const FLAT_KARATSUBA_INITIAL_LAYER: u32 = 3;

/// Tiny test polynomial initial layer parameter for the flat Karatsuba loop.
#[cfg(tiny_poly)]
#[cfg(any(test, feature = "benchmark"))]
pub const FLAT_KARATSUBA_INITIAL_LAYER: u32 = 2;

/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// All polynomials have maximum degree [`PolyConf::MAX_POLY_DEGREE`].
pub fn naive_cyclotomic_mul<C: PolyConf>(a: &Poly<C>, b: &Poly<C>) -> Poly<C> {
    debug_assert!(a.degree() <= C::MAX_POLY_DEGREE);
    debug_assert!(b.degree() <= C::MAX_POLY_DEGREE);

    let mut res: Poly<C> = a.naive_mul(b);

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

    #[allow(clippy::fn_to_numeric_cast_any)]
    {
        debug_assert_eq!(
            mod_poly_manual_mut::<C> as usize, mod_poly::<C> as usize,
            "this code assumes that mod_poly_manual_mut() is the fastest modulus function"
        );
    }
    debug_assert_eq!(res, mod_poly_ark_ref_slow(&dividend));

    assert!(res.degree() <= C::MAX_POLY_DEGREE);

    res
}

/// Returns `a * b` followed by reduction mod `XˆN + 1` using recursive Karatsuba method.
/// All polynomials have maximum degree [`PolyConf::MAX_POLY_DEGREE`].
///
/// # Performance
///
/// This implementation should be compiled in release mode without debug checks.
/// Some debug checks can slow it down by up to 25x:
/// ```text
/// debug = 2
/// debug-assertions = true
/// overflow-checks = true
/// ```
pub fn rec_karatsuba_mul<C: PolyConf>(a: &Poly<C>, b: &Poly<C>) -> Poly<C> {
    rec_karatsuba_mul_inner(a, b, C::MAX_POLY_DEGREE)
}

/// Returns `a * b` followed by reduction mod `XˆN + 1` using recursive Karatsuba method.
/// The returned polynomial has a degree less than or equal to `chunk`.
///
/// At each recursion level, polynomials start with maximum degree `chunk`, and are split to maximum degree `chunk/2`.
fn rec_karatsuba_mul_inner<C: PolyConf>(a: &Poly<C>, b: &Poly<C>, chunk: usize) -> Poly<C> {
    debug_assert!(
        a.degree() <= chunk,
        "a.degree() = {}, chunk = {chunk}",
        a.degree()
    );
    debug_assert!(
        b.degree() <= chunk,
        "b.degree() = {}, chunk = {chunk}",
        b.degree()
    );

    // invariant: the number of coefficients is a power of 2, before and after this function runs
    debug_assert_eq!(chunk.count_ones(), 1);
    const_assert_eq!(REC_KARATSUBA_MIN_DEGREE.count_ones(), 1);

    let mut res;

    // if a or b has degree less than min, condition is true
    let cond_a = a.degree() <= REC_KARATSUBA_MIN_DEGREE;
    let cond_b = b.degree() <= REC_KARATSUBA_MIN_DEGREE;
    let rec_cond = cond_a || cond_b;
    if rec_cond {
        // If degree is less than the recursion minimum, just use the naive version
        res = a.naive_mul(b);
    } else {
        // TODO: split this large code block into smaller functions, and benchmark the overall performance.
        // (Smaller functions can be inlined, and the compiler can optimize better.)

        // Otherwise recursively call for al.bl and ar.br
        let (mut al, ar) = poly_split_half(a, chunk);
        let (mut bl, br) = poly_split_half(b, chunk);

        let albl = rec_karatsuba_mul_inner(&al, &bl, chunk / 2);
        let arbr = rec_karatsuba_mul_inner(&ar, &br, chunk / 2);

        al += ar;
        let alpar = al;

        bl += br;
        let blpbr = bl;

        // Compute:
        // y = (al + ar).(bl + br)
        //   = al.bl + al.br + ar.bl + ar.br
        let mut y = rec_karatsuba_mul_inner(&alpar, &blpbr, chunk / 2);

        // Compute:
        // res = al.bl + (y - al.bl - ar.br)xˆn/2 + (ar.br)x^n
        //     = al.bl + (al.br + ar.bl)xˆn/2 + (ar.br)x^n
        // but in reverse order.

        // + (ar.br)x^n
        // This negates ar.br if n is equal to the max degree (terminating case),
        // and negates any terms over the max degree if n is slightly less (leading zeroes edge case).
        res = arbr.new_mul_xn(chunk);

        // + (y - al.bl - ar.br)xˆn/2
        y -= &albl;
        y -= arbr;

        // `res` will be reduced if needed, but that should only happen once in the first loop.
        y.mul_xn(chunk / 2);

        res += y;

        // + al.bl
        res += albl;

        debug_assert_eq!(res, naive_cyclotomic_mul(a, b), "\n{a:?}\n*\n{b:?}\n")
    }

    // If reduction isn't needed, this is very cheap.
    res.reduce_mod_poly();
    res
}

/// Returns `a * b` followed by reduction mod `XˆN + 1` using flat Karatsuba method.
/// The returned polynomial has a degree less than [`PolyConf::MAX_POLY_DEGREE`].
///
/// This implementation can be parallelized since for each layer
/// we have that chunks are independent of each other.
//
// TODO:
// - split the `for` and `while` loops into functions, and benchmark the overall performance.
// - split large code blocks into smaller functions, and benchmark the overall performance.
#[cfg(any(test, feature = "benchmark"))]
#[allow(clippy::cognitive_complexity)]
pub fn flat_karatsuba_mul<C: PolyConf>(a: &Poly<C>, b: &Poly<C>) -> Poly<C> {
    use std::ops::{Add, Sub};

    debug_assert!(a.degree() <= C::MAX_POLY_DEGREE);
    debug_assert!(b.degree() <= C::MAX_POLY_DEGREE);

    // The final number of layers in the flat Karatsuba `while` loop.
    // `FLAT_KARATSUBA_INITIAL_LAYER` skips some layers.
    let recursion_height: u32 = usize::ilog2(C::MAX_POLY_DEGREE);

    debug_assert!(FLAT_KARATSUBA_INITIAL_LAYER <= recursion_height);
    const_assert!(FLAT_KARATSUBA_INITIAL_LAYER > 1);

    // invariant: the number of coefficients is a power of 2
    debug_assert_eq!(C::MAX_POLY_DEGREE.count_ones(), 1);

    let mut first_layer_number = FLAT_KARATSUBA_INITIAL_LAYER;
    let mut chunk_size = 2usize.pow(first_layer_number - 1);
    let first_layer_length = C::MAX_POLY_DEGREE / chunk_size;
    let mut polys_current_layer: Vec<Poly<C>> = vec![];
    let mut polys_next_layer: Vec<Poly<C>> = vec![];
    let a_chunks = poly_split(a, chunk_size);
    let b_chunks = poly_split(b, chunk_size);

    debug_assert_eq!(a_chunks.len(), b_chunks.len());
    debug_assert_eq!(
        a_chunks.len(),
        C::MAX_POLY_DEGREE / chunk_size,
        "{} / {chunk_size}",
        C::MAX_POLY_DEGREE
    );

    // Take 2 at each step
    for i in 0..first_layer_length / 2 {
        // al, ar
        let al = &a_chunks[2 * i];
        let ar = &a_chunks[2 * i + 1];
        // bl, br
        let bl = &b_chunks[2 * i];
        let br = &b_chunks[2 * i + 1];

        let albl = al.naive_mul(bl);
        let mut arbr = ar.naive_mul(br);
        let alpar = al.add(ar);
        let blpbr = bl.add(br);
        // Compute y = (al + ar).(bl + br)
        let mut res = alpar.naive_mul(&blpbr);

        // Compute res = al.bl + (y - al.bl - ar.br)xˆ1 + (ar.br)x^2
        res = res.sub(&albl);
        res = res.sub(&arbr);
        res.mul_xn(chunk_size);
        res = res.add(albl);

        arbr.mul_xn(2 * chunk_size);
        res = res.add(arbr);

        polys_current_layer.push(res);
    }

    debug_assert_eq!(polys_current_layer.len() * 2, a_chunks.len());

    chunk_size *= 2;

    while first_layer_number < recursion_height {
        let a_chunks = poly_split(a, chunk_size);
        let b_chunks = poly_split(b, chunk_size);
        let layer_length = polys_current_layer.len();

        // Take 2
        debug_assert_eq!(a_chunks.len(), b_chunks.len());
        debug_assert_eq!(a_chunks.len(), polys_current_layer.len());
        debug_assert_eq!(
            a_chunks.len(),
            C::MAX_POLY_DEGREE / chunk_size,
            "{} / {chunk_size}",
            C::MAX_POLY_DEGREE
        );

        // Take two polynomials each round
        for j in 0..layer_length / 2 {
            // al, ar
            let al = &a_chunks[2 * j];
            let ar = &a_chunks[2 * j + 1];
            // bl, br
            let bl = &b_chunks[2 * j];
            let br = &b_chunks[2 * j + 1];

            let albl = &polys_current_layer[2 * j];
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

            let aux = arbr.new_mul_xn(2 * chunk_size);
            res = res.add(aux);

            polys_next_layer.push(res);
        }
        polys_current_layer = polys_next_layer;
        polys_next_layer = vec![];
        first_layer_number += 1;
        chunk_size *= 2;
    }

    debug_assert_eq!(polys_current_layer.len(), 1);
    let mut res = polys_current_layer.remove(0);
    // Just one final reduction is better than reducing along the computation
    res.reduce_mod_poly();

    debug_assert_eq!(res, naive_cyclotomic_mul(a, b), "\n{a:?}\n*\n{b:?}\n");

    res
}

/// Split the polynomial into `C::MAX_POLY_DEGREE / k` parts, in order from the constant term to the degree.
/// Any of the polynomials can be zero.
#[cfg(any(test, feature = "benchmark"))]
pub fn poly_split<C: PolyConf>(a: &Poly<C>, k: usize) -> Vec<Poly<C>> {
    // invariant: k must be a power of 2
    debug_assert_eq!(k.count_ones(), 1);

    let mut res: Vec<Poly<C>> = a
        .coeffs
        .chunks(k)
        .map(Poly::from_coefficients_slice)
        .collect();

    // Pad with zeroes if needed.
    res.resize(C::MAX_POLY_DEGREE / k, Poly::zero());

    res
}

/// Split the polynomial into left and right parts of size `chunk / 2`.
/// Either polynomial can be zero.
///
/// Returns `(low, high)`, where `low` contains the constant term.
///
/// All polynomials have maximum degree [`PolyConf::MAX_POLY_DEGREE`]. The modulus remains the same even after
/// the split.
pub fn poly_split_half<C: PolyConf>(a: &Poly<C>, chunk: usize) -> (Poly<C>, Poly<C>) {
    debug_assert!(chunk <= C::MAX_POLY_DEGREE);

    let (quotient, remainder) = a.new_div_xn(chunk / 2);

    (remainder, quotient)
}
