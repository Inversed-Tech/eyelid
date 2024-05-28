//! Plaintext iris matching tests.

use crate::{iris::conf::IrisConf, plaintext::is_iris_match};

pub mod gen;

pub mod matching;

/// Assert that iris comparison results are the same regardless of the order of the iris codes.
pub fn assert_iris_compare<C: IrisConf>(
    expected_result: bool,
    description: &str,
    eye_a: &C::IrisCode,
    mask_a: &C::IrisMask,
    eye_b: &C::IrisCode,
    mask_b: &C::IrisMask,
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
