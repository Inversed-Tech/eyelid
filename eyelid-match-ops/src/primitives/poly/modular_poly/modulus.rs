//! Reduction by the polynomial modulus `X^[C::MAX_POLY_DEGREE] + 1`.

use ark_ff::{One, Zero};
use ark_poly::polynomial::Polynomial;

use crate::primitives::poly::{Coeff, Poly};

// TODO: delete this after the search and replace.
use super::conf::PolyConf;
/// Temporary alias to make things compile.
pub const FULL_RES_POLY_DEGREE: usize = super::conf::TestRes::MAX_POLY_DEGREE;

/// The fastest available modular polynomial operation.
pub use mod_poly_manual_mut as mod_poly;

/// Reduces `dividend` to `dividend % [POLY_MODULUS]`.
///
/// This is the most efficient manual implementation.
pub fn mod_poly_manual_mut<C: PolyConf>(dividend: &mut Poly<C>) {
    let mut i = C::MAX_POLY_DEGREE;
    while i < dividend.coeffs.len() {
        let q = i / C::MAX_POLY_DEGREE;
        let r = i % C::MAX_POLY_DEGREE;

        // In the cyclotomic ring we have that XˆN = -1,
        // therefore all elements from N to 2N-1 are negated.
        //
        // For performance reasons, we use <Vec as IndexMut>,
        // because the loop condition limits `i` to valid indexes.
        if q % 2 == 1 {
            dividend.coeffs[r] = dividend.coeffs[r] - dividend.coeffs[i];
        } else {
            dividend.coeffs[r] = dividend.coeffs[r] + dividend.coeffs[i];
        }
        i += 1;
    }

    // The coefficients of C::MAX_POLY_DEGREE and higher have already been summed above.
    dividend.coeffs.truncate(C::MAX_POLY_DEGREE);

    // The coefficients could sum to zero, so make sure the polynomial is in the canonical form.
    dividend.truncate_to_canonical_form();
}

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This clones then uses the manual implementation.
#[cfg(inefficient)]
pub fn mod_poly_manual_ref<C: PolyConf>(dividend: &Poly<C>) -> Poly<C> {
    let mut dividend = dividend.clone();
    mod_poly_manual_mut(&mut dividend);
    dividend
}

/// Returns the remainder of `dividend % [POLY_MODULUS]`, as a polynomial.
///
/// This uses an [`ark-poly`] library implementation, which always creates a new polynomial.
pub fn mod_poly_ark_ref_slow<C: PolyConf>(dividend: &Poly<C>) -> Poly<C> {
    // The DenseOrSparsePolynomial implementation ensures canonical form.
    let (_quotient, remainder) = dividend
        .divide_with_q_and_r(&new_unreduced_poly_modulus_slow::<C>())
        .expect("POLY_MODULUS is not zero");

    remainder
}

/// Reduces `dividend` to `dividend % [POLY_MODULUS]`.
///
/// This uses an [`ark-poly`] library implementation, and entirely replaces the inner polynomial representation.
#[cfg(inefficient)]
pub fn mod_poly_ark_mut<C: PolyConf>(dividend: &mut Poly<C>) {
    let remainder = mod_poly_ark_ref(dividend);
    *dividend = remainder;
}

/// Constructs and returns a new polynomial modulus used for the polynomial field, `X^[C::MAX_POLY_DEGREE] + 1`.
/// This means that `X^[C::MAX_POLY_DEGREE] = -1`.
///
/// This is the canonical but un-reduced form of the modulus, because the reduced form is the zero polynomial.
///
/// TODO: work out how to generically make this a lazy static.
/// Crates like `interned` or `lazy_static` might help, but we'll have to expand their macros and make them generic.
pub fn new_unreduced_poly_modulus_slow<C: PolyConf>() -> Poly<C> {
    let mut poly = Poly::zero();

    // Since the leading coefficient is non-zero, this is in canonical form.
    // Resize to the maximum size first, to avoid repeated reallocations.
    poly[C::MAX_POLY_DEGREE] = Coeff::one();
    poly[0] = Coeff::one();

    // Check canonicity and degree.
    assert_eq!(poly.degree(), C::MAX_POLY_DEGREE);

    poly
}
