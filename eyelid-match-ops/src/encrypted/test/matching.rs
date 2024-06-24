//! Encrypted iris matching tests.

use crate::encoded::{PolyCode, PolyQuery};
use crate::encrypted::{EncryptedPolyCode, EncryptedPolyQuery};
use crate::iris::conf::IrisConf;
use crate::plaintext::test::matching::{different, matching};
use crate::primitives::yashe::Yashe;
use crate::{EncodeConf, FullBits, FullRes, PolyConf, YasheConf};
use colored::Colorize;

#[test]
fn test_matching_homomorphic_codes() {
    matching_codes::<FullBits>();
}

fn matching_codes<C: EncodeConf<PlainConf = FullRes>>()
where
    C::PlainConf: YasheConf,
    <C::PlainConf as PolyConf>::Coeff: From<u128> + From<u64> + From<i64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C::PlainConf> = Yashe::new();
    let (private_key, public_key) = ctx.keygen(&mut rng);

    for (description, eye_a, mask_a, eye_b, mask_b) in
        matching::<FullBits, { FullBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<FullBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);

        let encrypted_poly_query = EncryptedPolyQuery::convert_and_encrypt_query(
            ctx,
            poly_query.clone(),
            &public_key,
            &mut rng,
        );
        let encrypted_poly_code = EncryptedPolyCode::convert_and_encrypt_code(
            ctx,
            poly_code.clone(),
            &public_key,
            &mut rng,
        );

        let res = encrypted_poly_query
            .is_match(ctx, &private_key, &encrypted_poly_code)
            .expect("encrypted matching must work");
        assert!(
            res,
            "{description} must match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
        println!(
            "{} {description} {} ✅",
            "Encrypted encoding of matching iris codes indeed matches:"
                .cyan()
                .bold(),
            "OK".bright_blue().bold(),
        );
    }
}

/// Check different (non-matching) test cases.
#[test]
fn test_different_homomorphic_codes() {
    different_hom_codes::<FullBits>();
}

fn different_hom_codes<C: EncodeConf<PlainConf = FullRes>>()
where
    C::PlainConf: YasheConf,
    <C::PlainConf as PolyConf>::Coeff: From<u128> + From<u64> + From<i64>,
{
    let mut rng = rand::thread_rng();
    let ctx: Yashe<C::PlainConf> = Yashe::new();
    let (private_key, public_key) = ctx.keygen(&mut rng);

    for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<FullBits, { FullBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<FullBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code: PolyCode<FullBits> = PolyCode::from_plaintext(eye_b, mask_b);

        let encrypted_poly_query = EncryptedPolyQuery::convert_and_encrypt_query(
            ctx,
            poly_query.clone(),
            &public_key,
            &mut rng,
        );
        let encrypted_poly_code = EncryptedPolyCode::convert_and_encrypt_code(
            ctx,
            poly_code.clone(),
            &public_key,
            &mut rng,
        );

        let res = encrypted_poly_query
            .is_match(ctx, &private_key, &encrypted_poly_code)
            .expect("matching must work");
        assert!(
            !res,
            "{description} must not match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
        println!(
            "{} {description} {} ✅",
            "Encrypted encoding of different iris codes indeed doesn't match:"
                .cyan()
                .bold(),
            "OK".bright_blue().bold(),
        );
    }
}
