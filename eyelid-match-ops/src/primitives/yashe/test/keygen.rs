use crate::primitives::yashe::{inverse, Poly};
use crate::primitives::yashe::{Yashe, YasheParams};
use ark_ff::One;
use ark_poly::Polynomial;

#[allow(dead_code)]
fn keygen_helper<const MAX_POLY_DEGREE: usize>() {
    // TODO: how to deal with different sets of parameters?
    // We must be able to test all the different parameterizations
    let rng = rand::thread_rng();
    let params = YasheParams {
        t: 1024,
        _delta: 3.2,
    };
    let ctx: Yashe<MAX_POLY_DEGREE> = Yashe::new(params);
    let (private_key, public_key) = ctx.keygen(rng);

    let f_inv = inverse(&private_key.f);
    let priv_key_inv = inverse(&private_key.priv_key);
    assert_eq!(
        private_key.f * f_inv.expect("Polynomial f must be invertible"),
        Poly::one()
    );
    assert_eq!(
        private_key.priv_key * priv_key_inv.expect("Private key must be invertible"),
        Poly::one()
    );

    // TODO: test small coeff size
    assert_eq!(public_key.h.degree(), MAX_POLY_DEGREE - 1);
}

#[test]
fn test_keygen() {
    keygen_helper::<2048>();
}
