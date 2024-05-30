//! Plaintext iris matching tests.

use std::any::type_name;

use crate::{
    iris::conf::{IrisCode, IrisConf, IrisMask},
    plaintext::is_iris_match,
};

pub mod gen;

pub mod matching;

/// Assert that iris comparison results are the same regardless of the order of the iris codes.
pub fn assert_iris_compare<C: IrisConf, const STORE_ELEM_LEN: usize>(
    expected_result: bool,
    description: &str,
    eye_a: &IrisCode<STORE_ELEM_LEN>,
    mask_a: &IrisMask<STORE_ELEM_LEN>,
    eye_b: &IrisCode<STORE_ELEM_LEN>,
    mask_b: &IrisMask<STORE_ELEM_LEN>,
) {
    assert_eq!(
        expected_result,
        is_iris_match::<C, STORE_ELEM_LEN>(eye_a, mask_a, eye_b, mask_b),
        "{description}, test case order, {}\n\
        eye_a: {eye_a:?}\n\
        mask_a: {mask_a:?}\n\
        eye_b: {eye_b:?}\n\
        mask_b: {mask_b:?}\n\
        ",
        type_name::<C>(),
    );
    assert_eq!(
        expected_result,
        is_iris_match::<C, STORE_ELEM_LEN>(eye_b, mask_b, eye_a, mask_a),
        "{description}, reverse order, {}\n\
        eye_b: {eye_b:?}\n\
        mask_b: {mask_b:?}\n\
        eye_a: {eye_a:?}\n\
        mask_a: {mask_a:?}\n\
        ",
        type_name::<C>(),
    );
}
