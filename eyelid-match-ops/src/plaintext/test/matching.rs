//! Full match tests for plaintext iris codes and masks.

use crate::plaintext::test::gen::{set_iris_code, visible_iris_mask};

use super::assert_iris_compare;

/// Check a basic match case.
#[test]
fn matching_codes() {
    assert_iris_compare(
        true,
        &set_iris_code(),
        &visible_iris_mask(),
        &set_iris_code(),
        &visible_iris_mask(),
    );
}
