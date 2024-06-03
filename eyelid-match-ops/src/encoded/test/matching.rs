//! Tests for plaintext iris code matching.

use crate::{
    encoded::{PolyCode, PolyQuery},
    iris::conf::IrisConf,
    plaintext::test::matching::{different, matching},
    FullRes, IrisBits, TestRes,
};

/// Check matching test cases.
#[test]
fn matching_codes() {
    for (description, eye_a, mask_a, eye_b, mask_b) in
        matching::<TestRes, { TestRes::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<TestRes> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(res, "{} must match", description);
    }

    for (description, eye_a, mask_a, eye_b, mask_b) in
        matching::<IrisBits, { IrisBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<IrisBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(res, "{} must match", description);
    }
}

/// Check different (non-matching) test cases.
#[test]
fn different_codes() {
    // TODO: get this working with cfg(tiny_poly) and TestRes
    for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<FullRes, { FullRes::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<FullRes> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(!res, "{} must not match", description);
    }

    for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<IrisBits, { IrisBits::STORE_ELEM_LEN }>().iter()
    {
        let poly_query: PolyQuery<IrisBits> = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code).expect("matching must work");
        assert!(!res, "{} must not match", description);
    }
}
