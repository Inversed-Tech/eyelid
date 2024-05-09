//! Unit tests for Encryption and Decryption

use crate::primitives::yashe::{BigInt, Coeff, PolyConf, Yashe, YasheParams};
use crate::primitives::poly::TestRes;
use ark_ff::{BigInteger, PrimeField};

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
    let m_dec = ctx.decrypt(c, private_key);
    assert_eq!(m.m, m_dec.m);
}

#[test]
fn encrypt_decrypt_test() {
    encrypt_decrypt_helper::<TestRes>();
}

fn divq_helper<C: PolyConf>() {
    let mut rng = rand::thread_rng();
    let params = YasheParams {
        t: 1024,
        delta: 3.2,
    };
    let ctx: Yashe<C> = Yashe::new(params);

    let mut res = ctx.divq(BigInt::zero());
    assert_eq!(res, BigInt::zero());

    let qm1d2 = Coeff::MODULUS_MINUS_ONE_DIV_TWO; // q-1/2
    res = ctx.divq(qm1d2);
    assert_eq!(res, BigInt::zero());

    let mut val = Coeff::MODULUS; // q
    res = ctx.divq(val);
    assert_eq!(res, BigInt::one());

    let mut carry = val.add_with_carry(&qm1d2); // q + q-1/2
    res = ctx.divq(val);
    assert_eq!(res, BigInt::one());
    assert!(!carry); // no carry expected

    let mut two = BigInt::one();
    two.add_with_carry(&BigInt::one());

    carry = val.add_with_carry(&Coeff::MODULUS); // q + q-1/2 + q
    res = ctx.divq(val);
    assert_eq!(res, two);
    assert!(!carry); // no carry expected

    carry = val.add_with_carry(&qm1d2); // q + q-1/2 + q + q-1/2 < 3q (EDGE CASE)
    res = ctx.divq(val);
    assert_eq!(res, two);
    assert!(!carry); // no carry expected

    carry = val.add_with_carry(&BigInt::one()); // 3q (EDGE CASE)
    res = ctx.divq(val);
    assert_eq!(res, two);
    assert!(!carry); // no carry expected

    let mut three = two.clone();
    three.add_with_carry(&BigInt::one());

    carry = val.add_with_carry(&BigInt::one()); // 3q (EDGE CASE)
    res = ctx.divq(val);
    assert_eq!(res, three);
    assert!(!carry); // no carry expected

}

#[test]
fn divq_test() {
    divq_helper::<TestRes>();
}
