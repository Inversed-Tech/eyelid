//! Encrypted iris matching tests.

use crate::iris::conf::IrisConf;
use crate::encoded::{PolyCode, PolyQuery};
use crate::encrypted::{EncryptedPolyCode, EncryptedPolyQuery};
use crate::primitives::yashe::Yashe; 
use crate::plaintext::test::matching::{different, matching};
use crate::{EncodeConf, FullBits, FullRes, PolyConf, YasheConf};

#[test]
fn test_matching_codes() {
    matching_codes::<FullBits>();
    //matching_codes::<MiddleBits>();
}

fn matching_codes<C: EncodeConf<PlainConf = FullRes>>()
//fn matching_codes<C: EncodeConf>()
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

        //dbg!(poly_query.clone());
        //dbg!(poly_code.clone());
        
        // for each coefficient, if it is larger than C::PlainConf::modulus_minus_one_div_two_as_u128 then add C::PlainConf::T
        // otherwise do nothing
        for i in 0..poly_query.polys.len() {
            //let mut poly_query_polys = poly_query.polys[i].clone();
            //for j in 0..poly_query.polys.len() {
            #[allow(unused_mut)]
            for mut coeff in poly_query.polys[i].coeffs_mut() {
                let mut coeff_res: u128 = C::PlainConf::coeff_as_u128(*coeff);
                if coeff_res > <C::PlainConf as YasheConf>::modulus_minus_one_div_two_as_u128() {
                    coeff_res += u128::from(C::PlainConf::T);
                    *coeff = coeff_res.into();
                }
            }
        }
        // do the same  for poly_code
        for i in 0..poly_code.polys.len() {
            //let mut poly_code_polys = poly_code.polys[i].clone();
            #[allow(unused_mut)]
            for mut coeff in poly_code.polys[i].coeffs_mut() {
                let mut coeff_res: u128 = C::PlainConf::coeff_as_u128(*coeff);
                if coeff_res > <C::PlainConf as YasheConf>::modulus_minus_one_div_two_as_u128() {
                    coeff_res += u128::from(C::PlainConf::T);
                    *coeff = coeff_res.into();
                }
            }
        }

        //dbg!(poly_query.clone());
        //dbg!(poly_code.clone());
         
        let encrypted_poly_query = EncryptedPolyQuery::encrypt_query(ctx, poly_query.clone(), &public_key, &mut rng);
        let encrypted_poly_code = EncryptedPolyCode::encrypt_code(ctx, poly_code.clone(), &public_key, &mut rng);

        let res = encrypted_poly_query.is_match(ctx, private_key.clone(), &encrypted_poly_code).expect("encrypted matching must work");
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
fn test_different_codes() {
    different_codes::<FullBits>();
    //matching_codes::<MiddleBits>();
}

fn different_codes<C: EncodeConf<PlainConf = FullRes>>()
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
        let mut poly_code = PolyCode::from_plaintext(eye_b, mask_b);

        //dbg!(poly_query.clone());
        //dbg!(poly_code.clone());
        
        // for each coefficient, if it is larger than C::PlainConf::modulus_minus_one_div_two_as_u128 then add C::PlainConf::T
        // to get a value in the range [0, T-1], otherwise do nothing
        for i in 0..poly_query.polys.len() {
            //let mut poly_query_polys = poly_query.polys[i].clone();
            //for j in 0..poly_query.polys.len() {
            #[allow(unused_mut)]
            for mut coeff in poly_query.polys[i].coeffs_mut() {
                let mut coeff_res: u128 = C::PlainConf::coeff_as_u128(*coeff);
                if coeff_res > <C::PlainConf as YasheConf>::modulus_minus_one_div_two_as_u128() {
                    coeff_res += u128::from(C::PlainConf::T);
                    *coeff = coeff_res.into();
                }
            }
        }
        // do the same  for poly_code
        for i in 0..poly_code.polys.len() {
            //let mut poly_code_polys = poly_code.polys[i].clone();
            #[allow(unused_mut)]
            for mut coeff in poly_code.polys[i].coeffs_mut() {
                let mut coeff_res: u128 = C::PlainConf::coeff_as_u128(*coeff);
                if coeff_res > <C::PlainConf as YasheConf>::modulus_minus_one_div_two_as_u128() {
                    coeff_res += u128::from(C::PlainConf::T);
                    *coeff = coeff_res.into();
                }
            }
        }

        //dbg!(poly_query.clone());
        //dbg!(poly_code.clone());

        let encrypted_poly_query = EncryptedPolyQuery::encrypt_query(ctx, poly_query.clone(), &public_key, &mut rng);
        let encrypted_poly_code = EncryptedPolyCode::encrypt_code(ctx, poly_code.clone(), &public_key, &mut rng);

        let res = encrypted_poly_query.is_match(ctx, private_key.clone(), &encrypted_poly_code).expect("matching must work");
        assert!(
            !res,
            "{description} must not match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
    }
}
