//! Iris test data generation.

use rand::Rng;

use crate::plaintext::{IrisCode, IrisMask};

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

/// Returns an iris code with no bits set.
pub fn unset_iris_code() -> IrisCode {
    IrisCode::ZERO
}

/// Returns an iris code with all bits set.
pub fn set_iris_code() -> IrisCode {
    IrisCode::ZERO
}

/// Returns an iris mask that is totally occluded.
pub fn occluded_iris_mask() -> IrisMask {
    // We don't have type safety so this works for now.
    unset_iris_code()
}

/// Returns an iris mask that is fully visible (no occlusions).
pub fn visible_iris_mask() -> IrisMask {
    unset_iris_code()
}
