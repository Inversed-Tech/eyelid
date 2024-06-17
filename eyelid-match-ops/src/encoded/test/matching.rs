//! Tests for plaintext iris code matching.

use crate::{
    encoded::{PolyCode, PolyQuery}, iris::conf::IrisConf, plaintext::test::matching::{different, matching}, FullBits, MiddleBits, TestBits,
};

/// Check matching test cases.
#[test]
fn matching_codes() {
    for (description, eye_a, mask_a, eye_b, mask_b) in
        matching::<TestBits, { TestBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<TestBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(
            res,
            "{description} must match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
    }

    for (description, eye_a, mask_a, eye_b, mask_b) in
        matching::<MiddleBits, { MiddleBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<MiddleBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code).expect("matching must work");
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
fn different_codes() {
    // TODO: get this working with cfg(tiny_poly) and TestBits

    /*for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<MiddleBits, { MiddleBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<MiddleBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);

        // for each coefficient, if it is larger than C::PlainConf::modulus_minus_one_div_two_as_u128 then add C::PlainConf::T
        // to get a value in the range [0, T-1], otherwise do nothing
        /*for i in 0..poly_query.polys.len() {
            //let mut poly_query_polys = poly_query.polys[i].clone();
            #[allow(unused_mut)]
            for mut coeff in poly_query.polys[i].coeffs_mut() {
                let mut coeff_res: u128 = <MiddleBits as EncodeConf>::PlainConf::coeff_as_u128(*coeff);
                if coeff_res > <<MiddleBits as EncodeConf>::PlainConf as YasheConf>::modulus_minus_one_div_two_as_u128() {
                    coeff_res += u128::from(<MiddleBits as EncodeConf>::PlainConf::T);
                    *coeff = coeff_res.into();
                }
            }
            poly_query.polys[i].truncate_to_canonical_form();
        }

        // do the same for poly_code
        for i in 0..poly_code.polys.len() {
            #[allow(unused_mut)]
            for mut coeff in poly_code.polys[i].coeffs_mut() {
                let mut coeff_res: u128 = <MiddleBits as EncodeConf>::PlainConf::coeff_as_u128(*coeff);
                if coeff_res > <<MiddleBits as EncodeConf>::PlainConf as YasheConf>::modulus_minus_one_div_two_as_u128() {
                    coeff_res += u128::from(<MiddleBits as EncodeConf>::PlainConf::T);
                    *coeff = coeff_res.into();
                }
            }
            poly_code.polys[i].truncate_to_canonical_form();
        }*/

        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(
            !res,
            "{description} must not match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
    }*/

    for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<FullBits, { FullBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<FullBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);

        /*for i in 0..poly_query.polys.len() {
            //let mut poly_query_polys = poly_query.polys[i].clone();
            #[allow(unused_mut)]
            for mut coeff in poly_query.polys[i].coeffs_mut() {
                let mut coeff_res: u128 = <FullBits as EncodeConf>::PlainConf::coeff_as_u128(*coeff);
                if coeff_res > <<FullBits as EncodeConf>::PlainConf as YasheConf>::modulus_minus_one_div_two_as_u128() {
                    coeff_res += u128::from(<FullBits as EncodeConf>::PlainConf::T);
                    *coeff = coeff_res.into();
                }
            }
            poly_query.polys[i].truncate_to_canonical_form();
        }

        // do the same for poly_code
        for i in 0..poly_code.polys.len() {
            #[allow(unused_mut)]
            for mut coeff in poly_code.polys[i].coeffs_mut() {
                let mut coeff_res: u128 = <FullBits as EncodeConf>::PlainConf::coeff_as_u128(*coeff);
                if coeff_res > <<FullBits as EncodeConf>::PlainConf as YasheConf>::modulus_minus_one_div_two_as_u128() {
                    coeff_res += u128::from(<FullBits as EncodeConf>::PlainConf::T);
                    *coeff = coeff_res.into();
                }
            }
            poly_code.polys[i].truncate_to_canonical_form();
        }*/

        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(
            !res,
            "{description} must not match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
    }
}
