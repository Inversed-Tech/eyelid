//! Polynomial inverse.
use crate::primitives::poly::{Coeff, Poly};
use ark_ff::{Field, One, Zero};
use ark_poly::Polynomial;

/// Returns the primitive polynomial which is the inverse of `a` in the
/// cyclotomic ring, if it exists. Otherwise, returns an error.
///
/// Implementation based on Algorithm 3.3.1 (Page 118) from
/// "A Course in Computational Algebraic Number Theory", Henri Cohen.
/// We don't divide by content of `a` and `b` every time,
/// just in the end of the algorithm.
///
/// When `d` is a constant polynomial and `a` is the polynomial modulus
/// (which reduces to `0`), we have that `b/cont(d)` is the primitive
/// multiplicative inverse of `y`.
pub fn inverse<C: PolyConf>(
    a: &Poly<C>,
) -> Result<Poly<C>, &'static str> {
    let unreduced_mod_pol = Poly::new_unreduced_poly_modulus_slow();

    let (_x, y, d) = extended_gcd(&unreduced_mod_pol, a);

    // If `d` is a non-zero constant, we can compute the inverse of `d`,
    // and calculate the final primitive inverse.
    if d.is_zero() {
        Err("Can't invert the zero polynomial")
    } else if d.degree() > 0 {
        Err("Non-invertible polynomial")
    } else {
        // Reduce to a primitive polynomial.
        let mut inv: Poly<C> = y;
        // Compute the inverse of the content
        let content_inv: Coeff = d[0].inverse().expect("just checked for zero");
        // Divide by `content_inv`
        inv *= content_inv;

        Ok(inv)
    }
}

/// Helps to calculate the equation `cur = prev - q.cur`.
fn update_diophantine<C: PolyConf>(
    mut prev: Poly<C>,
    cur: Poly<C>,
    q: &Poly<C>,
) -> (Poly<C>, Poly<C>) {
    let mul_res = &cur * q;
    let new_prev = cur;

    prev -= mul_res;
    let new_cur = prev;

    (new_cur, new_prev)
}

/// Returns polynomials `(x, y, d)` such that `a.x + b.y = d`.
pub fn extended_gcd<C: PolyConf>(
    a: &Poly<C>,
    b: &Poly<C>,
) -> (
    Poly<C>,
    Poly<C>,
    Poly<C>,
) {
    // Invariant a.xi + b.yi = ri

    // init with x0=1, y0=0, r0=a
    let mut x_prev: Poly<C> = Poly::one();
    let mut y_prev = Poly::zero();
    let mut ri_prev = a.clone();
    // next:     x1=0, y1=1, r1=b
    let mut x_cur = Poly::zero();
    let mut y_cur = Poly::one();
    let mut ri_cur = b.clone();

    let mut q: Poly<C>;

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
