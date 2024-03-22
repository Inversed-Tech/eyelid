//! Cyclotomic polynomial operations using ark-poly

use ark_ff::{Fp128, MontBackend, MontConfig, One, Zero};
use ark_poly::polynomial::{
    univariate::{DenseOrSparsePolynomial, DensePolynomial},
    Polynomial,
};
use lazy_static::lazy_static;
use std::ops::Add;
use std::ops::Sub;

/// The maximum exponent in the polynomial.
pub const MAX_POLY_DEGREE: usize = 2048;
/// Minimum degree for recursive Karatsuba calls
pub const MIN_KARATSUBA_REC_DEGREE: usize = 8; // TODO: fine tune

/// The configuration of the modular field used for polynomial coefficients.
#[derive(MontConfig)]
#[modulus = "93309596432438992665667"]
#[generator = "5"]
pub struct Fq79Config;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
/// t = 2ˆ15, q = 2ˆ79, N = 2048
//
// Sage commands:
// random_prime(2**79)
// 93309596432438992665667
// ff = GF(93309596432438992665667)
// ff.multiplicative_generator()
// 5
//
// We could also consider generating primes dynamically, but this could impact performance.
pub type Fq79 = Fp128<MontBackend<Fq79Config, 2>>;

/// A modular polynomial with coefficients in [`Fq79`],
/// and maximum degree [`MAX_POLY_DEGREE`].
//
// TODO: replace this with a type wrapper that uses the constant degree above.
pub type Poly = DensePolynomial<Fq79>;

lazy_static! {
    /// The polynomial modulus used for the polynomial field, `X^[MAX_POLY_DEGREE] + 1`.
    /// This means that `X^[MAX_POLY_DEGREE] = -1`.
    pub static ref POLY_MODULUS: DenseOrSparsePolynomial<'static, Fq79> = {
        let mut poly = zero_poly(MAX_POLY_DEGREE);

        poly[MAX_POLY_DEGREE] = Fq79::one();
        poly[0] = Fq79::one();

        assert_eq!(poly.degree(), MAX_POLY_DEGREE);

        poly.into()
    };
}

/// Returns the zero polynomial with `degree`.
///
/// This is not the canonical form, but it's useful for creating other polynomials.
/// (Non-canonical polynomials will panic when `degree()` is called on them.)
pub fn zero_poly(degree: usize) -> Poly {
    assert!(degree <= MAX_POLY_DEGREE);

    let mut poly = Poly::zero();
    poly.coeffs = vec![Fq79::zero(); degree + 1];
    poly
}

/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn naive_cyclotomic_mul(a: &Poly, b: &Poly) -> Poly {
    // TODO: change these assertions to debug_assert!() to avoid panics in production code.
    assert!(a.degree() <= MAX_POLY_DEGREE);
    assert!(b.degree() <= MAX_POLY_DEGREE);

    let mut res = a.naive_mul(b);

    for i in 0..MAX_POLY_DEGREE {
        // In the cyclotomic ring we have that XˆN = -1,
        // therefore all elements from N to 2N-1 are negated.
        if i + MAX_POLY_DEGREE < res.coeffs.len() {
            res[i] = res[i] - res[i + MAX_POLY_DEGREE];
        };
    }

    // These elements have already been negated and summed above.
    res.coeffs.truncate(MAX_POLY_DEGREE);

    // Leading elements might be zero, so make sure the polynomial is in the canonical form.
    while res.coeffs.last() == Some(&Fq79::zero()) {
        res.coeffs.pop();
    }

    assert!(res.degree() <= MAX_POLY_DEGREE);

    res
}

/// Returns `a * b` followed by reduction mod `XˆN + 1`.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn karatsuba_mul(a: &Poly, b: &Poly) -> Poly {
    // General idea:
    // if a or b has degree less than min, condition is true

    let n = a.degree()+1;
    //dbg!(n);

    let cond_a = a.degree()+1 < MIN_KARATSUBA_REC_DEGREE;
    let cond_b = b.degree()+1 < MIN_KARATSUBA_REC_DEGREE;
    let rec_cond = cond_a || cond_b;
    // If degree is less than the recursion minimum, just use the naive version
    let mut res = zero_poly(n);
    if rec_cond {
        res = naive_cyclotomic_mul(a, b);
    } else {
        // Otherwise recursively call for al.bl and ar.br
        // Compute al, ar
        let (al, ar) = poly_split(a);
        // Compute bl, br
        let (bl, br) = poly_split(b);
        // Compute al.bl
        let albl = karatsuba_mul(&al, &bl);
        // Compute ar.br
        let arbr = karatsuba_mul(&ar, &br);
        // Compute (al + ar)
        let alpar = al.add(ar);
        // Compute (bl + br)
        let blpbr = bl.add(br);
        // Compute y = (al + ar).(bl + br)
        let y = karatsuba_mul(&alpar, &blpbr);
        // Compute res = (al.bl - ar.br) + (y - al.bl - ar.br)xˆn/2
        res = y.clone();
        res = res.sub(&albl);
        res = res.sub(&arbr);
        let halfn = n/2;
        let mut xnb2 = zero_poly(halfn);
        xnb2.coeffs[halfn] = Fq79::one();
        dbg!(xnb2.clone());
        res = naive_cyclotomic_mul(&res.clone(), &xnb2);
        res = res.add(albl);
        res = res.sub(&arbr);
    };
    res.coeffs.truncate(MAX_POLY_DEGREE);
    // Leading elements might be zero, so make sure the polynomial is in the canonical form.
    while res.coeffs.last() == Some(&Fq79::zero()) {
        res.coeffs.pop();
    }
    res
}

pub fn poly_split(a: &Poly) -> (Poly, Poly) {
    // TODO: replace naive/inefficient implementation
    // Starting with naive code is easy to code and to understand the algorithm
    let n = a.degree()+1;
    let halfn = n/2;
    let mut al = zero_poly(halfn);
    let mut ar = zero_poly(halfn);
    for i in 0..halfn {
        al.coeffs[i] = a.coeffs[i];
        ar.coeffs[i] = a.coeffs[halfn + i];
    }
    al.coeffs.truncate(MAX_POLY_DEGREE);
    while al.coeffs.last() == Some(&Fq79::zero()) {
        al.coeffs.pop();
    }
    ar.coeffs.truncate(MAX_POLY_DEGREE);
    while ar.coeffs.last() == Some(&Fq79::zero()) {
        ar.coeffs.pop();
    }
    (al, ar)
}

// TODO: put tests in another file to speed up compilation.

/// Generates a cyclotomic polynomial of `degree`, with random coefficients in Fq79.
/// `degree` must be less than or equal to [`MAX_POLY_DEGREE`].
///
/// In rare cases, the degree can be less than `degree`,
/// because the random coefficient of `X^[MAX_POLY_DEGREE]` is zero.
#[cfg(any(test, feature = "benchmark"))]
pub fn rand_poly(degree: usize) -> Poly {
    use ark_poly::DenseUVPolynomial;
    use rand::thread_rng;

    assert!(degree <= MAX_POLY_DEGREE);

    // We can't use test_rng() here, because a deterministic RNG can make benchmarks inaccurate.
    let mut rng = thread_rng();

    // TODO: consider using a random degree, biased towards small and large degree edge cases.
    let poly = Poly::rand(degree, &mut rng);

    assert!(poly.degree() <= degree);

    poly
}

/// Test cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_cyclotomic_mul_rand() {
    let p1 = rand_poly(MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MAX_POLY_DEGREE - 1);
    }

    // Xˆ{N-1}, multiplying by it will rotate by N-1 and negate (except the first).
    let mut xnm1 = zero_poly(MAX_POLY_DEGREE - 1);
    xnm1.coeffs[MAX_POLY_DEGREE - 1] = Fq79::one();

    assert_eq!(xnm1.degree(), MAX_POLY_DEGREE - 1);

    let res = naive_cyclotomic_mul(&p1, &xnm1);
    assert!(res.degree() <= MAX_POLY_DEGREE);

    for i in 0..MAX_POLY_DEGREE - 1 {
        // Negative numbers are automatically converted to canonical
        // representation in the interval [0, Fq79Config::MODULUS)
        assert_eq!(res[i], -p1[i + 1]);
    }
    assert_eq!(res[MAX_POLY_DEGREE - 1], p1[0]);

    // Zero coefficients aren't stored.
    if res.degree() >= MAX_POLY_DEGREE {
        for i in (MAX_POLY_DEGREE)..=res.degree() {
            assert_eq!(res[i], Fq79::zero());
        }
    }
}

/// Test cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_karatsuba_mul_rand() {
    let p1 = rand_poly(MAX_POLY_DEGREE - 1);
    let p2 = rand_poly(MAX_POLY_DEGREE - 1);

    #[allow(clippy::int_plus_one)]
    {
        assert!(p1.degree() <= MAX_POLY_DEGREE - 1);
        assert!(p2.degree() <= MAX_POLY_DEGREE - 1);
    }

    let expected = naive_cyclotomic_mul(&p1, &p2);
    assert!(expected.degree() <= MAX_POLY_DEGREE);
    let res = karatsuba_mul(&p1, &p2);
    assert!(res.degree() <= MAX_POLY_DEGREE);

    assert_eq!(expected, res);
}

/// Test cyclotomic multiplication that results in `X^[MAX_POLY_DEGREE]`.
#[test]
fn test_cyclotomic_mul_max_degree() {
    use ark_poly::DenseUVPolynomial;

    // X^MAX_POLY_DEGREE
    let mut x_max = zero_poly(MAX_POLY_DEGREE);
    x_max[MAX_POLY_DEGREE] = Fq79::one();

    // There is a shorter representation of X^N as the constant `Fq79Config::MODULUS - 1`.
    let x_max = DenseOrSparsePolynomial::from(x_max);
    let (q, x_max) = x_max
        .divide_with_q_and_r(&*POLY_MODULUS)
        .expect("is divisible by X^MAX_POLY_DEGREE");

    assert_eq!(q, Poly::from_coefficients_vec(vec![Fq79::one()]));
    assert_eq!(
        x_max,
        // TODO: should this be a constant?
        Poly::from_coefficients_vec(vec![Fq79::zero() - Fq79::one()]),
    );

    for i in 0..=MAX_POLY_DEGREE {
        // X^i * X^{MAX_POLY_DEGREE - i} = X^MAX_POLY_DEGREE
        let mut p1 = zero_poly(i);
        p1[i] = Fq79::one();

        let mut p2 = zero_poly(MAX_POLY_DEGREE - i);
        p2[MAX_POLY_DEGREE - i] = Fq79::one();

        assert_eq!(p1.degree() + p2.degree(), MAX_POLY_DEGREE);

        let res = naive_cyclotomic_mul(&p1, &p2);

        // Make sure it's X^N
        assert_eq!(res, x_max);
    }
}
