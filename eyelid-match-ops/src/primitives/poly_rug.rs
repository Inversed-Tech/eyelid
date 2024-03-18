//! [`rug_polynomial`]/FLINT implementation of basic polynomial types and operations.

use std::sync::Arc;

use lazy_static::lazy_static;
use rug::Integer;
use rug_polynomial::ModPoly;

/// The maximum exponent in the polynomial.
pub const MAX_POLY_DEGREE: usize = 2048;

/// The maximum length of the polynomial.
/// One more than [`MAX_POLY_DEGREE`].
pub const MAX_POLY_LEN: usize = MAX_POLY_DEGREE + 1;

// We define a finite field using pre-computed primes and generators.
// These are the parameters for full resolution, according to the Inversed Tech report.
lazy_static! {
    /// The modular field used for polynomial coefficients.
    /// t = 2ˆ15, q = 2ˆ79, N = 2048
    // Sage commands:
    // random_prime(2**79)
    // 93309596432438992665667
    // ff = GF(93309596432438992665667)
    // ff.multiplicative_generator()
    // 5
    // We could also consider generating primes dynamically, but this could impact performance.
    pub static ref COEFFICIENT_MODULUS: Arc<Integer> = {
        let coeff = 93309596432438992665667_i128.into();
        Arc::new(coeff)
    };
}

// Work around ModPoly not being thread-safe.
thread_local! {
    /// The polynomial modulus used for the polynomial field, `X^[MAX_POLY_DEGREE] + 1`.
    /// This means that `X^[MAX_POLY_DEGREE] = -1`.
    pub static POLY_MODULUS: Polynomial = {
        let mut poly = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
        poly.set_coefficient_ui(MAX_POLY_DEGREE, 1);
        poly.set_coefficient_ui(0, 1);
        poly
    }
}

/// A modular polynomial with coefficients in [`COEFFICIENT_MODULUS`],
/// and maximum degree [`MAX_POLY_DEGREE`].
//
// TODO: replace this with a type wrapper that uses the constant moduli and degree above.
pub type Polynomial = ModPoly;

/// Returns `a*b % [POLY_MODULUS]`, with positive coefficients.
/// The returned polynomial has maximum degree [`MAX_POLY_DEGREE`].
pub fn cyclotomic_mul(a: &Polynomial, b: &Polynomial) -> Polynomial {
    assert!(a.len() <= MAX_POLY_LEN);
    assert!(b.len() <= MAX_POLY_LEN);

    let mut res = a.clone();
    res *= b;

    // TODO: benchmark the manual cyclotomic_mul impl, to see if it's faster.
    POLY_MODULUS.with(|m| res %= m);

    assert!(res.len() <= MAX_POLY_LEN);

    res
}

// TODO: put tests in another file to speed up compilation.

/// Returns a random polynomial with degree `degree`,
/// which must be less than or equal to [`MAX_POLY_DEGREE`].
///
/// In rare cases, the degree can be less than `degree`,
/// because the random coefficient of `X^[MAX_POLY_DEGREE]` is zero.
pub fn rand_poly(degree: usize) -> Polynomial {
    assert!(degree <= MAX_POLY_DEGREE);

    let mut rng = rug::rand::RandState::new();

    // TODO: consider using a random degree, biased towards small and large degree edge cases.
    let mut poly = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
    for i in 0..=degree {
        let coeff: Integer = COEFFICIENT_MODULUS.random_below_ref(&mut rng).into();
        poly.set_coefficient(i, &coeff);
    }
    // TODO: create a degree() method that correctly subtracts one
    assert!(poly.len() <= degree + 1);

    poly
}

/// Test cyclotomic multiplication of a random polynomial by `X^{[MAX_POLY_DEGREE] - 1}`.
#[test]
fn test_cyclotomic_mul_rand() {
    // Create a random polynomial.
    let p1 = rand_poly(MAX_POLY_DEGREE - 1);

    // Multiplying by Xˆ{MAX_POLY_DEGREE-1} will rotate by MAX_POLY_DEGREE-1 and negate
    // (except for X^{MAX_POLY_DEGREE-1} and X^MAX_POLY_DEGREE)
    let mut xnm1 = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
    xnm1.set_coefficient_ui(MAX_POLY_DEGREE - 1, 1);
    assert_eq!(xnm1.len(), MAX_POLY_LEN - 1);

    let res = cyclotomic_mul(&p1, &xnm1);
    assert!(res.len() <= MAX_POLY_LEN);

    for i in 0..MAX_POLY_DEGREE - 1 {
        assert_eq!(
            res.get_coefficient(i),
            // The coefficients from cyclotomic_mul() are positive modulo COEFFICIENT_MODULUS
            COEFFICIENT_MODULUS.as_ref() - p1.get_coefficient(i + 1)
        );
    }
    assert_eq!(
        res.get_coefficient(MAX_POLY_DEGREE - 1),
        // TODO: is this constant some function of COEFFICIENT_MODULUS?
        62264161555756135262324_i128,
    );
    assert_eq!(
        res.get_coefficient(MAX_POLY_DEGREE),
        p1.get_coefficient(MAX_POLY_DEGREE)
    );
}

/// Test cyclotomic multiplication that results in `X^[MAX_POLY_DEGREE]`.
#[test]
fn test_cyclotomic_mul_max_degree() {
    // X^MAX_POLY_DEGREE
    let mut x_max = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
    x_max.set_coefficient_ui(MAX_POLY_DEGREE, 1);
    // There is a shorter representation of -1 as the constant `COEFFICIENT_MODULUS - 1`.
    POLY_MODULUS.with(|m| x_max %= m);
    assert_eq!(
        x_max,
        Polynomial::from_int(
            COEFFICIENT_MODULUS.as_ref().clone(),
            Integer::from(COEFFICIENT_MODULUS.as_ref() - 1),
        )
    );

    for i in 0..=MAX_POLY_DEGREE {
        // X^i * X^{MAX_POLY_DEGREE - i} = X^MAX_POLY_DEGREE
        let mut p1 = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
        p1.set_coefficient_ui(i, 1);

        let mut p2 = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
        p2.set_coefficient_ui(MAX_POLY_DEGREE - i, 1);

        // TODO: create a degree() method that correctly subtracts one
        assert!(p1.len() <= MAX_POLY_LEN);
        assert!(p2.len() <= MAX_POLY_LEN);
        assert_eq!(p1.len() + p2.len(), MAX_POLY_DEGREE + 2);

        let res = cyclotomic_mul(&p1, &p2);

        // Make sure it's X^N
        assert_eq!(res, x_max);
    }
}
