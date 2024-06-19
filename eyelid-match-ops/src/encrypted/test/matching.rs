//! Encrypted iris matching tests.

use crate::encoded::conf::LargeRes;
use crate::encoded::{PolyCode, PolyQuery};
use crate::encrypted::{EncryptedPolyCode, EncryptedPolyQuery};
use crate::iris::conf::IrisConf;
use crate::plaintext::test::matching::{different, matching};
use crate::primitives::poly::Poly;
use crate::primitives::yashe::Yashe;
use crate::{EncodeConf, FullBits, PolyConf, YasheConf};

// Given a vector of polynomails, for each coefficient, if it is larger than Q-1/2 then add T.
// Otherwise do nothing.
fn convert_negative_coefficients<C: EncodeConf<PlainConf = LargeRes>>(
    poly_vec: &mut Vec<Poly<C::PlainConf>>,
) {
    for i in 0..poly_vec.len() {
        Poly::coeffs_modify_non_zero(
            &mut poly_vec[i],
            |coeff: &mut <C::PlainConf as PolyConf>::Coeff| {
                let mut coeff_res = C::PlainConf::coeff_as_big_int(*coeff);
                if coeff_res > <C::PlainConf as YasheConf>::modulus_minus_one_div_two_as_big_int() {
                    coeff_res += C::PlainConf::T;
                    *coeff = C::PlainConf::big_int_as_coeff(coeff_res);
                }
            },
        );
    }
}

#[test]
fn test_matching_homomorphic_codes() {
    matching_codes::<FullBits>();
}

fn matching_codes<C: EncodeConf<PlainConf = LargeRes>>()
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
        let mut poly_query: PolyQuery<FullBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let mut poly_code = PolyCode::from_plaintext(eye_b, mask_b);

        convert_negative_coefficients::<C>(&mut poly_query.polys);
        convert_negative_coefficients::<C>(&mut poly_code.polys);

        let encrypted_poly_query =
            EncryptedPolyQuery::encrypt_query(ctx, poly_query.clone(), &public_key, &mut rng);
        let encrypted_poly_code =
            EncryptedPolyCode::encrypt_code(ctx, poly_code.clone(), &public_key, &mut rng);

        let res = encrypted_poly_query
            .is_match(ctx, private_key.clone(), &encrypted_poly_code)
            .expect("encrypted matching must work");
        assert!(
            res,
            "{description} must match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
    }
}

/// Check different (non-matching) test cases.
#[test]
fn test_different_homomorphic_codes() {
    different_hom_codes::<FullBits>();
}

fn different_hom_codes<C: EncodeConf<PlainConf = LargeRes>>()
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
        let mut poly_query: PolyQuery<FullBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let mut poly_code: PolyCode<FullBits> = PolyCode::from_plaintext(eye_b, mask_b);

        convert_negative_coefficients::<C>(&mut poly_query.polys);
        convert_negative_coefficients::<C>(&mut poly_code.polys);

        let encrypted_poly_query =
            EncryptedPolyQuery::encrypt_query(ctx, poly_query.clone(), &public_key, &mut rng);
        let encrypted_poly_code =
            EncryptedPolyCode::encrypt_code(ctx, poly_code.clone(), &public_key, &mut rng);

        let res = encrypted_poly_query
            .is_match(ctx, private_key.clone(), &encrypted_poly_code)
            .expect("matching must work");
        assert!(
            !res,
            "{description} must not match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
    }
}
