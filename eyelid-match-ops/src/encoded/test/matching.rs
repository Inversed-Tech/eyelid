//! Tests for plaintext iris code matching.

use crate::{
    encoded::{PolyCode, PolyQuery},
    iris::conf::IrisConf,
    plaintext::test::matching::{different, matching},
    FullBits, MiddleBits, TestBits,
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

    for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<MiddleBits, { MiddleBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<MiddleBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(
            !res,
            "{description} must not match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
    }

    for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<FullBits, { FullBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<FullBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(
            !res,
            "{description} must not match:\n\
            query: {poly_query:?}\n\
            code: {poly_code:?}"
        );
    }
}
