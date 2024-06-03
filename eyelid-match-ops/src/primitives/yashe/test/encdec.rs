//! Unit tests for Encryption and Decryption

use crate::primitives::{
    poly::modular_poly::conf::{FullRes, IrisBits},
    yashe::{Yashe, YasheConf},
};

fn encrypt_decrypt_helper<C: YasheConf>()
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C> = Yashe::new();

    let (private_key, public_key) = ctx.keygen(&mut rng);
    let m = ctx.sample_message(&mut rng);
    let c = ctx.encrypt(m.clone(), public_key, &mut rng);
    let m_dec = ctx.decrypt(c.clone(), private_key);

    assert_eq!(m, m_dec);
}

#[test]
fn encrypt_decrypt_test() {
    // The TinyTest config doesn't work for encryption, so we test full resolution,
    // and a large polynomial with the same number of terms as the number of iris bits.
    // Testing multiple configs is important for code coverage, and to check for hard-coded assumptions.
    //
    // TODO: find a config that does work and use it for TestRes/TinyPoly.
    encrypt_decrypt_helper::<FullRes>();
    encrypt_decrypt_helper::<IrisBits>();
}
