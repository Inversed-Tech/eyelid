//! Full match tests for plaintext iris codes and masks.

use lazy_static::lazy_static;

use crate::plaintext::{
    test::gen::{
        occluded_iris_mask, random_iris_code, rotate_not_too_much, rotate_too_much, set_iris_code,
        similar_iris_code, unset_iris_code, visible_iris_mask, CODES, MASKS,
    },
    IrisCode, IrisMask,
};

#[cfg(test)]
use super::assert_iris_compare;

lazy_static! {
    /// Mask combinations which are always occluded.
    pub static ref OCCLUDED: Vec<(String, IrisMask, IrisMask)> = {
        let mut occluded = Vec::new();

        for (description, mask) in MASKS.iter() {
            occluded.push((format!("occluded, {description}"), occluded_iris_mask(), *mask));
            occluded.push((format!("{description}, occluded"), *mask, occluded_iris_mask()));
        }

        occluded
    };

    /// Test cases which always match.
    pub static ref MATCHING: Vec<(String, IrisCode, IrisMask, IrisCode, IrisMask)> = {
        let same_rand = random_iris_code();
        let iris2 = similar_iris_code(&same_rand);
        let iris3 = rotate_not_too_much(&same_rand);

        let mut matching = vec![
            ("set, visible".to_string(), set_iris_code(), visible_iris_mask(), set_iris_code(), visible_iris_mask()),
            ("unset, visible".to_string(), unset_iris_code(), visible_iris_mask(), unset_iris_code(), visible_iris_mask()),
            ("same rand, visible".to_string(), same_rand, visible_iris_mask(), same_rand, visible_iris_mask()),
            ("similar".to_string(), same_rand, visible_iris_mask(), iris2, visible_iris_mask()),
            ("not too much rotated".to_string(), same_rand, visible_iris_mask(), iris3, visible_iris_mask()),
        ];

        // These cases technically match, but only because the numbers of matching and visible
        // bits are both zero
        for (mask_description, mask_a, mask_b) in OCCLUDED.iter() {
            for (eye_a_description, eye_a) in CODES.iter() {
                for (eye_b_description, eye_b) in CODES.iter() {
                    matching.push(
                        (
                            format!("{eye_a_description}, {eye_b_description}, {mask_description}"),
                            *eye_a,
                            *mask_a,
                            *eye_b,
                            *mask_b,
                         )
                    );
                }
            }
        }

        matching
    };

    /// Test cases which never match.
    pub static ref DIFFERENT: Vec<(String, IrisCode, IrisMask, IrisCode, IrisMask)> = {
        let same_rand = random_iris_code();
        let iris2 = random_iris_code();
        let iris3 = rotate_too_much(&iris2);

        vec![
            ("set/unset, visible".to_string(), set_iris_code(), visible_iris_mask(), unset_iris_code(), visible_iris_mask()),
            ("inverted rand, visible".to_string(), same_rand, visible_iris_mask(), !same_rand, visible_iris_mask()),
            ("different".to_string(), same_rand, visible_iris_mask(), iris2, visible_iris_mask()),
            ("too much rotated".to_string(), iris2, visible_iris_mask(), iris3, visible_iris_mask()),
        ]
    };
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
