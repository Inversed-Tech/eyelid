//! Full match tests for plaintext iris codes and masks.

use crate::{
    iris::conf::IrisConf,
    plaintext::test::gen::{
        codes, masks, occluded_iris_mask, random_iris_code, rotate_not_too_much, rotate_too_much,
        set_iris_code, similar_iris_code, unset_iris_code, visible_iris_mask,
    },
};

#[cfg(test)]
use super::assert_iris_compare;

/// Returns a list of mask combinations which are always occluded.
pub fn occluded<C: IrisConf>() -> Vec<(String, C::IrisMask, C::IrisMask)> {
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
pub fn matching<C: IrisConf>() -> Vec<(String, C::IrisCode, C::IrisMask, C::IrisCode, C::IrisMask)>
{
    let same_rand = random_iris_code::<C>();
    let iris2 = similar_iris_code::<C>(&same_rand);
    let iris3 = rotate_not_too_much::<C>(&same_rand);

    let mut matching = vec![
        (
            "set, visible".to_string(),
            set_iris_code::<C>(),
            visible_iris_mask::<C>(),
            set_iris_code::<C>(),
            visible_iris_mask::<C>(),
        ),
        (
            "unset, visible".to_string(),
            unset_iris_code::<C>(),
            visible_iris_mask::<C>(),
            unset_iris_code::<C>(),
            visible_iris_mask::<C>(),
        ),
        (
            "same rand, visible".to_string(),
            same_rand,
            visible_iris_mask::<C>(),
            same_rand,
            visible_iris_mask::<C>(),
        ),
        (
            "similar".to_string(),
            same_rand,
            visible_iris_mask::<C>(),
            iris2,
            visible_iris_mask::<C>(),
        ),
        (
            "not too much rotated".to_string(),
            same_rand,
            visible_iris_mask::<C>(),
            iris3,
            visible_iris_mask::<C>(),
        ),
    ];

    // These cases technically match, but only because the numbers of matching and visible
    // bits are both zero
    for (mask_description, mask_a, mask_b) in occluded::<C>().iter() {
        for (eye_a_description, eye_a) in codes::<C>().iter() {
            for (eye_b_description, eye_b) in codes::<C>().iter() {
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
pub fn different<C: IrisConf>() -> Vec<(String, C::IrisCode, C::IrisMask, C::IrisCode, C::IrisMask)>
{
    let same_rand = random_iris_code();
    let iris2 = random_iris_code();
    let iris3 = rotate_too_much(&iris2);

    vec![
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
        (
            "different".to_string(),
            same_rand,
            visible_iris_mask(),
            iris2,
            visible_iris_mask(),
        ),
        (
            "too much rotated".to_string(),
            iris2,
            visible_iris_mask(),
            iris3,
            visible_iris_mask(),
        ),
    ]
}

/// Check matching test cases.
#[test]
fn matching_codes() {
    for (description, eye_a, mask_a, eye_b, mask_b) in MATCHING.iter() {
        assert_iris_compare(true, description, eye_a, mask_a, eye_b, mask_b);
    }
}

/// Check different (non-matching) test cases.
#[test]
fn different_codes() {
    for (description, eye_a, mask_a, eye_b, mask_b) in DIFFERENT.iter() {
        assert_iris_compare(false, description, eye_a, mask_a, eye_b, mask_b);
    }
}
