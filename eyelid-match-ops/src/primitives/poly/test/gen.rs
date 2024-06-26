//! Test data generation for polynomials.

use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use rand::Rng;

use crate::primitives::poly::{Poly, PolyConf};

/// Returns an un-reduced cyclotomic polynomial of `degree`, with random coefficients in [`PolyConf::Coeff`].
/// `degree` must be less than or equal to [`PolyConf::MAX_POLY_DEGREE`].
///
/// In rare cases, the degree can be less than `degree`,
/// because the random coefficient of `X^[C::MAX_POLY_DEGREE]` is zero.
pub fn rand_poly<C: PolyConf>(degree: usize) -> Poly<C> {
    use rand::thread_rng;

    // We can't use test_rng() here, because a deterministic RNG can make benchmarks inaccurate.
    let mut rng = thread_rng();

    // TODO: consider using a random degree, biased towards small and large degree edge cases.
    let poly = Poly::rand(degree, &mut rng);

    assert!(poly.degree() <= degree);

    poly
}

impl<C: PolyConf> Poly<C> {
    // Shadow DenseUVPolynomial methods, but only make the method available in test code.

    /// Returns a random polynomial with degree `d`.
    /// Only for use in tests and benchmarks.
    pub fn rand<R: Rng>(d: usize, rng: &mut R) -> Self {
        DensePolynomial::rand(d, rng).into()
    }
}
