//! Iris test data generation.

use lazy_static::lazy_static;
use rand::Rng;

use crate::plaintext::{IrisCode, IrisMask};

lazy_static! {
    /// A list of all codes. Random codes are only listed once.
    /// The random data is different for each execution of the test program.
    pub static ref CODES: Vec<(&'static str, IrisCode)> = {
        vec![
            ("set", set_iris_code()),
            ("unset", unset_iris_code()),
            ("random", random_iris_code()),
        ]
    };

    /// A list of all masks. Random masks are only listed once.
    /// The random data is different for each execution of the test program.
    pub static ref MASKS: Vec<(&'static str, IrisMask)> = {
        vec![
            ("visible", visible_iris_mask()),
            ("occluded", occluded_iris_mask()),
            ("random", random_iris_mask()),
        ]
    };
}

/// Returns an iris code with uniformly random bits.
pub fn random_iris_code() -> IrisCode {
    let mut code = IrisCode::ZERO;
    let mut rng = rand::thread_rng();

    rng.fill(code.data.as_mut_slice());

    code
}

/// Returns an iris mask with uniformly random bits.
pub fn random_iris_mask() -> IrisMask {
    // We don't have type safety so this works for now.
    random_iris_code()
}

/// Returns an iris code with all bits set.
pub fn set_iris_code() -> IrisCode {
    !IrisCode::ZERO
}

/// Returns an iris code with no bits set.
pub fn unset_iris_code() -> IrisCode {
    IrisCode::ZERO
}

/// Returns an iris mask that is fully visible (no occlusions).
pub fn visible_iris_mask() -> IrisMask {
    set_iris_code()
}

/// Returns an iris mask that is totally occluded.
pub fn occluded_iris_mask() -> IrisMask {
    // We don't have type safety so this works for now.
    unset_iris_code()
}
