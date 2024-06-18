//! Unit tests for homomorphic operations

use std::any::type_name;

use crate::{
    encoded::conf::LargeRes,
    primitives::yashe::{Yashe, YasheConf},
    FullRes, MiddleRes,
};

fn homomorphic_addition_helper<C: YasheConf>()
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C> = Yashe::new();

    let (private_key, public_key) = ctx.keygen(&mut rng);
    let m1 = ctx.sample_message(&mut rng);
    let m2 = ctx.sample_message(&mut rng);
    let c1 = ctx.encrypt(m1.clone(), &public_key.clone(), &mut rng);
    let c2 = ctx.encrypt(m2.clone(), &public_key, &mut rng);
    let m = ctx.plaintext_add(m1, m2);
    let c = ctx.ciphertext_add(c1, c2);
    // Additions can be regularly decrypted using the private key
    let m_dec = ctx.decrypt(c.clone(), &private_key);

    assert_eq!(m, m_dec, "addition test failed for {}", type_name::<C>());
}

fn homomorphic_multiplication_helper_negative<C: YasheConf>()
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C> = Yashe::new();

    let (private_key, public_key) = ctx.keygen(&mut rng);
    let m1 = ctx.sample_message(&mut rng);
    let m2 = ctx.sample_message(&mut rng);
    let c1 = ctx.encrypt(m1.clone(), &public_key.clone(), &mut rng);
    let c2 = ctx.encrypt(m2.clone(), &public_key, &mut rng);
    let m = ctx.plaintext_mul(m1, m2);
    let c = ctx.ciphertext_mul(c1, c2);
    // Multiplications can't be regularly decrypted using the private key
    let m_dec = ctx.decrypt(c.clone(), &private_key);

    assert_ne!(
        m,
        m_dec,
        "negative multiplication test failed for {}",
        type_name::<C>()
    );
}

// Positive multiplication test for generic messages
fn homomorphic_multiplication_helper_positive<C: YasheConf>()
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C> = Yashe::new();

    let (private_key, public_key) = ctx.keygen(&mut rng);
    let m1 = ctx.sample_message(&mut rng);
    let m2 = ctx.sample_message(&mut rng);
    let c1 = ctx.encrypt(m1.clone(), &public_key.clone(), &mut rng);
    let c2 = ctx.encrypt(m2.clone(), &public_key, &mut rng);
    let m = ctx.plaintext_mul(m1, m2);
    let c = ctx.ciphertext_mul(c1, c2);
    let m_dec = ctx.decrypt_mul(c.clone(), &private_key);

    assert_eq!(
        m,
        m_dec,
        "positive multiplication test failed for {}",
        type_name::<C>()
    );
}

// Positive muiltiplication test for ternary messages, i.e. using sample_ternary_message
fn homomorphic_multiplication_helper_positive_ternary<C: YasheConf>()
where
    C::Coeff: From<u128> + From<u64> + From<i64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C> = Yashe::new();

    let (private_key, public_key) = ctx.keygen(&mut rng);
    let m1 = ctx.sample_ternary_message(&mut rng);
    let m2 = ctx.sample_ternary_message(&mut rng);
    let c1 = ctx.encrypt(m1.clone(), &public_key.clone(), &mut rng);
    let c2 = ctx.encrypt(m2.clone(), &public_key, &mut rng);
    let m = ctx.plaintext_mul(m1, m2);
    let c = ctx.ciphertext_mul(c1, c2);
    let m_dec = ctx.decrypt_mul(c.clone(), &private_key);

    assert_eq!(
        m,
        m_dec,
        "positive multiplication test failed for {}",
        type_name::<C>()
    );
}

// TODO: get these tests working with TestRes

#[test]
fn homomorphic_addition_test() {
    // Testing multiple configs is important for code coverage, and to check for hard-coded assumptions.
    // TODO: get TinyTest working in this module
    homomorphic_addition_helper::<MiddleRes>();
    homomorphic_addition_helper::<FullRes>();
    homomorphic_addition_helper::<LargeRes>();
}

#[test]
fn homomorphic_negative_multiplication_test() {
    homomorphic_multiplication_helper_negative::<MiddleRes>();
    homomorphic_multiplication_helper_negative::<FullRes>();
    homomorphic_multiplication_helper_negative::<LargeRes>();
}

#[test]
fn homomorphic_positive_multiplication_test() {
    homomorphic_multiplication_helper_positive::<MiddleRes>();
    homomorphic_multiplication_helper_positive_ternary::<MiddleRes>();
    homomorphic_multiplication_helper_positive::<FullRes>();
    homomorphic_multiplication_helper_positive_ternary::<FullRes>();
    homomorphic_multiplication_helper_positive::<LargeRes>();
    homomorphic_multiplication_helper_positive_ternary::<LargeRes>();
}
