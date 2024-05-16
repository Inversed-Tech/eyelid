//! Unit tests for Key Generation

use crate::primitives::{
    poly::{Poly, TestRes},
    yashe::{Yashe, YasheConf},
};
use ark_ff::One;
use ark_poly::Polynomial;

/// Auxiliary function for testing key generation
fn keygen_helper<C: YasheConf>()
where
    C::Coeff: From<i64> + From<u64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C> = Yashe::new();
    let (private_key, public_key) = ctx.keygen(&mut rng);

    let priv_key_inv = private_key.priv_key.inverse();

    assert_eq!(
        private_key.f[0] * C::t_as_coeff() + C::Coeff::one(),
        private_key.priv_key[0]
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
