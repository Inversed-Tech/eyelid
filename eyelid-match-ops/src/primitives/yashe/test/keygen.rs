//! Unit tests for Key Generation

use crate::primitives::{
    poly::TestRes,
    yashe::{Coeff, Poly, PolyConf, Yashe, YasheParams},
};
use ark_ff::One;
use ark_poly::Polynomial;

/// Auxiliary function for testing key generation
fn keygen_helper<C: PolyConf>() {
    // TODO: how to deal with different sets of parameters?
    // We must be able to test all the different parameterizations
    let mut rng = rand::thread_rng();
    let params = YasheParams {
        t: 1024,
        delta: 3.2,
    };
    let ctx: Yashe<C> = Yashe::new(params);
    let (private_key, public_key) = ctx.keygen(&mut rng);

    let f_inv = private_key.f.inverse();
    let priv_key_inv = private_key.priv_key.inverse();

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

    assert!(public_key.h.degree() < C::MAX_POLY_DEGREE);
}

#[test]
fn test_keygen() {
    keygen_helper::<TestRes>();
}
