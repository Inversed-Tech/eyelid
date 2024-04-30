//! Unit tests for Key Generation

use crate::primitives::{
    poly::FULL_RES_POLY_DEGREE,
    yashe::{inverse, Coeff, Poly, Yashe, YasheParams},
};
use ark_ff::One;
use ark_poly::Polynomial;

/// Auxiliary function for testing key generation
fn keygen_helper<const MAX_POLY_DEGREE: usize>() {
    // TODO: how to deal with different sets of parameters?
    // We must be able to test all the different parameterizations
    let rng = rand::thread_rng();
    let params = YasheParams {
        t: 1024,
        delta: 3.2,
    };
    let ctx: Yashe<MAX_POLY_DEGREE> = Yashe::new(params.clone());
    let (private_key, public_key) = ctx.keygen(rng);

    let f_inv = inverse(&private_key.f);
    let priv_key_inv = inverse(&private_key.priv_key);

    //dbg!(private_key.priv_key[0].clone());
    assert_eq!(
        private_key.f[0] * Coeff::from(params.t) + Coeff::one(),
        private_key.priv_key[0]
    );

    assert_eq!(
        private_key.f * f_inv.expect("Polynomial f must be invertible"),
        Poly::one()
    );
    assert_eq!(
        private_key.priv_key * priv_key_inv.expect("Private key must be invertible"),
        Poly::one()
    );

    assert_eq!(public_key.h.degree(), MAX_POLY_DEGREE - 1);
}

#[test]
fn test_keygen() {
    keygen_helper::<FULL_RES_POLY_DEGREE>();
}
