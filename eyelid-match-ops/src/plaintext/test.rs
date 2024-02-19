//! Plaintext iris matching tests.

use crate::plaintext::is_iris_match;

use super::{IrisCode, IrisMask};

pub mod gen;

mod matching;

/// Assert that iris comparison results are the same regardless of the order of the iris codes.
pub fn assert_iris_compare(
    expected_result: bool,
    description: &str,
    eye_a: &IrisCode,
    mask_a: &IrisMask,
    eye_b: &IrisCode,
    mask_b: &IrisMask,
) {
    assert_eq!(
        expected_result,
        is_iris_match(eye_a, mask_a, eye_b, mask_b),
        "{description}: test case order",
    );
    assert_eq!(
        expected_result,
        is_iris_match(eye_b, mask_b, eye_a, mask_a),
        "{description}: reverse order",
    );
}
