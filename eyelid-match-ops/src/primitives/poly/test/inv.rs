//! Tests for polynomial inverse.

use super::super::*;

#[test]
fn test_inverse() {
    let f = sample();
    // REMARK: For our parameter choices it is very likely to find
    // the inverse in the first attempt.
    // For small degree and coefficient modulus, the situation may change.
    let out = inverse(&f);
    assert!(out.is_ok());
}
