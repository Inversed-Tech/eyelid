//! Unit tests for Encryption and Decryption

use crate::primitives::poly::TestRes;
use crate::primitives::yashe::{PolyConf, Yashe, YasheParams};

fn encrypt_decrypt_helper<C: PolyConf>() {
    let mut rng = rand::thread_rng();
    let params = YasheParams {
        t: 1024,
        delta: 3.2,
    };
    let ctx: Yashe<C> = Yashe::new(params);
    let (private_key, public_key) = ctx.keygen(&mut rng);
    let m = ctx.sample_message(&mut rng);
    let c = ctx.encrypt(m.clone(), public_key, &mut rng);
    let m_dec = ctx.decrypt(c.clone(), private_key);

    assert_eq!(m.m, m_dec.m);
}

#[test]
fn encrypt_decrypt_test() {
    encrypt_decrypt_helper::<TestRes>();
}
