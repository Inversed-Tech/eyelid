//! Test data generation for polynomials.

use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use rand::Rng;

use crate::primitives::poly::Poly;

/// Returns an un-reduced cyclotomic polynomial of `degree`, with random coefficients in [`Coeff`].
/// `degree` must be less than or equal to [`MAX_POLY_DEGREE`].
///
/// In rare cases, the degree can be less than `degree`,
/// because the random coefficient of `X^[MAX_POLY_DEGREE]` is zero.
pub fn rand_poly(degree: usize) -> Poly {
    use rand::thread_rng;

    // We can't use test_rng() here, because a deterministic RNG can make benchmarks inaccurate.
    let mut rng = thread_rng();

    // TODO: consider using a random degree, biased towards small and large degree edge cases.
    let poly = Poly::rand(degree, &mut rng);

    assert!(poly.degree() <= degree);

    poly
}

impl Poly {
    // Shadow DenseUVPolynomial methods, but only make the method available in test code.

    /// Returns a random polynomial with degree `d`.
    /// Only for use in tests and benchmarks.
    pub fn rand<R: Rng>(d: usize, rng: &mut R) -> Self {
        DensePolynomial::rand(d, rng).into()
    }
}
