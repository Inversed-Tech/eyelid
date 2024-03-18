//! [`rug_polynomial`]/FLINT implementation of basic polynomial types and operations.

use std::sync::Arc;

use lazy_static::lazy_static;
use rug::Integer;
use rug_polynomial::ModPoly;

/// The maximum exponent in the polynomial.
pub const MAX_POLY_DEGREE: usize = 2048;

/// The maximum length of the polynomial.
/// One more than the degree.
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
    /// The polynomial modulus used for the polynomial field: `X^N + 1`.
    /// This means that `X^N = -1`.
    pub static POLY_MODULUS: Polynomial = {
        let mut poly = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
        poly.set_coefficient_ui(MAX_POLY_DEGREE, 1);
        poly.set_coefficient_ui(0, 1);
        poly
    }
}

/// A modular polynomial with coefficients in [`COEFFICIENT_MODULUS`],
/// and maximun degree [`MAX_POLY_DEGREE`].
//
// TODO: replace this with a type wrapper that uses the constant moduli and degree above.
pub type Polynomial = ModPoly;

pub fn cyclotomic_mul(a: Polynomial, b: Polynomial) -> Polynomial {
    assert!(a.len() <= MAX_POLY_LEN);
    assert!(b.len() <= MAX_POLY_LEN);

    let mut res = a * b;

    // TODO: benchmark the manual cyclotomic_mul impl, to see if it's faster.
    POLY_MODULUS.with(|m| res %= m);

    assert!(res.len() <= MAX_POLY_LEN);

    res
}

// TODO: put tests in another file to speed up compilation.
#[test]
fn test_cyclotomic_mul() {
    use rug::rand::RandState;

    let mut rng = RandState::new();

    // A random polynomial with degree `MAX_POLY_DEGREE - 1`
    let mut p1 = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
    for i in 0..MAX_POLY_DEGREE {
        let coeff: Integer = COEFFICIENT_MODULUS.random_below_ref(&mut rng).into();
        p1.set_coefficient(i, &coeff);
    }
    // TODO: create a degree() method that correctly subtracts one
    assert_eq!(p1.len(), MAX_POLY_LEN - 1);

    // Multiplying by Xˆ{N-1} will rotate by N-1 and negate (except for X^{N-1} and X^N)
    let mut xnm1 = Polynomial::new(COEFFICIENT_MODULUS.as_ref().clone());
    xnm1.set_coefficient_ui(MAX_POLY_DEGREE - 1, 1);
    assert_eq!(xnm1.len(), MAX_POLY_LEN - 1);

    let res = cyclotomic_mul(p1.clone(), xnm1);
    assert!(res.len() <= MAX_POLY_LEN);

    for i in 0..MAX_POLY_DEGREE - 1 {
        // TODO: fix cyclotomic_mul() so coefficients are positive
        assert_eq!(
            res.get_coefficient(i),
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
