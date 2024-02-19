//! Full match tests for plaintext iris codes and masks.

use lazy_static::lazy_static;

use crate::plaintext::{
    test::gen::{set_iris_code, unset_iris_code, visible_iris_mask},
    IrisCode, IrisMask,
};

use super::assert_iris_compare;

lazy_static! {
    /// Test cases which always match.
    static ref MATCHING: Vec<(&'static str, IrisCode, IrisMask, IrisCode, IrisMask)> = {
        vec![
            ("visible, set", set_iris_code(), visible_iris_mask(), set_iris_code(), visible_iris_mask()),
            ("visible, unset", unset_iris_code(), visible_iris_mask(), unset_iris_code(), visible_iris_mask()),
        ]
    };
}

/// Check a basic match case.
#[test]
fn matching_codes() {
    for (description, eye_a, mask_a, eye_b, mask_b) in MATCHING.iter() {
        assert_iris_compare(true, description, eye_a, mask_a, eye_b, mask_b);
    }
}
