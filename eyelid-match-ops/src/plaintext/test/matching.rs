//! Full match tests for plaintext iris codes and masks.

use crate::{
    iris::conf::{IrisCode, IrisConf, IrisMask},
    plaintext::test::gen::{
        codes, masks, occluded_iris_mask, random_iris_code, rotate_not_too_much, set_iris_code,
        similar_iris_code, unset_iris_code, visible_iris_mask,
    },
};

#[cfg(test)]
use super::assert_iris_compare;

/// Returns a list of mask combinations which are always occluded.
pub fn occluded<const STORE_ELEM_LEN: usize>(
) -> Vec<(String, IrisMask<STORE_ELEM_LEN>, IrisMask<STORE_ELEM_LEN>)> {
    let mut occluded = Vec::new();

    for (description, mask) in masks().iter() {
        occluded.push((
            format!("occluded, {description}"),
            occluded_iris_mask(),
            *mask,
        ));
        occluded.push((
            format!("{description}, occluded"),
            *mask,
            occluded_iris_mask(),
        ));
    }

    occluded
}

/// Returns test cases which always match.
pub fn matching<C: IrisConf, const STORE_ELEM_LEN: usize>() -> Vec<(
    String,
    IrisCode<STORE_ELEM_LEN>,
    IrisMask<STORE_ELEM_LEN>,
    IrisCode<STORE_ELEM_LEN>,
    IrisMask<STORE_ELEM_LEN>,
)> {
    let same_rand = random_iris_code();
    let iris2 = similar_iris_code(&same_rand);
    let iris3 = rotate_not_too_much::<C, STORE_ELEM_LEN>(&same_rand);

    let mut matching = vec![
        (
            "set, visible".to_string(),
            set_iris_code(),
            visible_iris_mask(),
            set_iris_code(),
            visible_iris_mask(),
        ),
        (
            "unset, visible".to_string(),
            unset_iris_code(),
            visible_iris_mask(),
            unset_iris_code(),
            visible_iris_mask(),
        ),
        (
            "same rand, visible".to_string(),
            same_rand,
            visible_iris_mask(),
            same_rand,
            visible_iris_mask(),
        ),
        (
            "similar".to_string(),
            same_rand,
            visible_iris_mask(),
            iris2,
            visible_iris_mask(),
        ),
        (
            "not too much rotated".to_string(),
            same_rand,
            visible_iris_mask(),
            iris3,
            visible_iris_mask(),
        ),
    ];

    // These cases technically match, but only because the numbers of matching and visible
    // bits are both zero
    for (mask_description, mask_a, mask_b) in occluded().iter() {
        for (eye_a_description, eye_a) in codes().iter() {
            for (eye_b_description, eye_b) in codes().iter() {
                matching.push((
                    format!("{eye_a_description}, {eye_b_description}, {mask_description}"),
                    *eye_a,
                    *mask_a,
                    *eye_b,
                    *mask_b,
                ));
            }
        }
    }

    matching
}

/// Returns a list of test cases which never match.
pub fn different<C: IrisConf, const STORE_ELEM_LEN: usize>() -> Vec<(
    String,
    IrisCode<STORE_ELEM_LEN>,
    IrisMask<STORE_ELEM_LEN>,
    IrisCode<STORE_ELEM_LEN>,
    IrisMask<STORE_ELEM_LEN>,
)> {
    let same_rand = random_iris_code();

    #[allow(unused_mut)]
    let mut res = vec![
        (
            "set/unset, visible".to_string(),
            set_iris_code(),
            visible_iris_mask(),
            unset_iris_code(),
            visible_iris_mask(),
        ),
        (
            "inverted rand, visible".to_string(),
            same_rand,
            visible_iris_mask(),
            !same_rand,
            visible_iris_mask(),
        ),
    ];

    // In small polynomials these tests can fail by chance.
    #[cfg(not(tiny_poly))]
    {
        use crate::plaintext::test::gen::rotate_too_much;

        let iris2 = random_iris_code();
        let iris3 = rotate_too_much::<C, STORE_ELEM_LEN>(&iris2);

        res.push((
            "different".to_string(),
            same_rand,
            visible_iris_mask(),
            iris2,
            visible_iris_mask(),
        ));
        #[cfg(not(tiny_poly))]
        res.push((
            "too much rotated".to_string(),
            iris2,
            visible_iris_mask(),
            iris3,
            visible_iris_mask(),
        ));
    }

    res
}

/// Check matching test cases.
#[test]
fn matching_codes() {
    use crate::{IrisBits, TestRes};

    for (description, eye_a, mask_a, eye_b, mask_b) in
        matching::<TestRes, { TestRes::STORE_ELEM_LEN }>().iter()
    {
        assert_iris_compare::<TestRes, { TestRes::STORE_ELEM_LEN }>(
            true,
            description,
            eye_a,
            mask_a,
            eye_b,
            mask_b,
        );
    }

    for (description, eye_a, mask_a, eye_b, mask_b) in
        matching::<IrisBits, { IrisBits::STORE_ELEM_LEN }>().iter()
    {
        assert_iris_compare::<IrisBits, { IrisBits::STORE_ELEM_LEN }>(
            true,
            description,
            eye_a,
            mask_a,
            eye_b,
            mask_b,
        );
    }
}

/// Check different (non-matching) test cases.
#[test]
fn different_codes() {
    use crate::{IrisBits, TestRes};

    for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<TestRes, { TestRes::STORE_ELEM_LEN }>().iter()
    {
        assert_iris_compare::<TestRes, { TestRes::STORE_ELEM_LEN }>(
            false,
            description,
            eye_a,
            mask_a,
            eye_b,
            mask_b,
        );
    }

    for (description, eye_a, mask_a, eye_b, mask_b) in
        different::<IrisBits, { IrisBits::STORE_ELEM_LEN }>().iter()
    {
        assert_iris_compare::<IrisBits, { IrisBits::STORE_ELEM_LEN }>(
            false,
            description,
            eye_a,
            mask_a,
            eye_b,
            mask_b,
        );
    }
}
