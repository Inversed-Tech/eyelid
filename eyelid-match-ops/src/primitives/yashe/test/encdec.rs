//! Unit tests for Encryption and Decryption

fn encrypt_decrypt_helper<C: PolyConf>() {
    let mut rng = rand::thread_rng();
    let params = YasheParams {
        t: 1024,
        delta: 3.2,
    };
    let ctx: Yashe<C> = Yashe::new(params);
    let (private_key, public_key) = ctx.keygen(&mut rng);
    let c = ctx.encrypt(m, public_key);
    let m_dec = ctx.decrypt(c, private_key);
    assert_eq!(m, m_dec);
}

fn encrypt_decrypt_test() {
    encrypt_decrypt_helper::<TestRes>();
}
