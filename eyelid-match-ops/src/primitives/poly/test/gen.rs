//! Test data generation for polynomials.

use super::super::*;

/// Returns a cyclotomic polynomial of `degree`, with random coefficients in Fq79.
/// `degree` must be less than or equal to [`MAX_POLY_DEGREE`].
///
/// In rare cases, the degree can be less than `degree`,
/// because the random coefficient of `X^[MAX_POLY_DEGREE]` is zero.
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
