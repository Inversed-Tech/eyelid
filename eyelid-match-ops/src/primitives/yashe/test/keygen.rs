//! Unit tests for Key Generation

use crate::primitives::{
    poly::TestRes,
    yashe::{Poly, PolyConf, Yashe},
};
use ark_ff::One;
use ark_poly::Polynomial;

/// Auxiliary function for testing key generation
fn keygen_helper<C: PolyConf>()
where
    C::Coeff: From<i64> + From<u64>,
{
    // TODO: how to deal with different sets of parameters?
    // We must be able to test all the different parameterizations
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C> = Yashe::new();
    let (private_key, public_key) = ctx.keygen(&mut rng);

    let f_inv = private_key.f.inverse();
    let priv_key_inv = private_key.priv_key.inverse();

    //dbg!(private_key.priv_key[0].clone());
    assert_eq!(
        private_key.f[0] * C::Coeff::from(params.t) + C::Coeff::one(),
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
