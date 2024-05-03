use crate::{
    encoded::{PolyCode, PolyQuery},
    plaintext::test::matching::{DIFFERENT, MATCHING},
};

/// Check matching test cases.
#[test]
fn matching_codes() {
    for (description, eye_a, mask_a, eye_b, mask_b) in MATCHING.iter() {
        let poly_query = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code);
        assert_eq!(true, res, "{} must match", description);
    }
}

/// Check different (non-matching) test cases.
#[test]
fn different_codes() {
    for (description, eye_a, mask_a, eye_b, mask_b) in DIFFERENT.iter() {
        let poly_query = PolyQuery::from_plaintext(eye_a, mask_a);
        let poly_code = PolyCode::from_plaintext(eye_b, mask_b);
        let res = poly_query.is_match(&poly_code);
        assert_eq!(false, res, "{} must not match", description);
    }
}
