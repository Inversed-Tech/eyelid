//! Unit tests for Encryption and Decryption

use std::any::type_name;

use crate::{
    primitives::yashe::{Yashe, YasheConf},
    MiddleRes, TestRes,
};

fn encrypt_decrypt_helper<C: YasheConf>()
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C> = Yashe::new();

    let (private_key, public_key) = ctx.keygen(&mut rng);
    let m = ctx.sample_message(&mut rng);
    let c = ctx.encrypt(m.clone(), &public_key, &mut rng);
    let m_dec = ctx.decrypt(c.clone(), &private_key);

    assert_eq!(m, m_dec, "{}", type_name::<C>());
}

#[test]
fn encrypt_decrypt_test() {
    // Testing multiple configs is important for code coverage, and to check for hard-coded assumptions.
    encrypt_decrypt_helper::<TestRes>();
    encrypt_decrypt_helper::<MiddleRes>();
}
