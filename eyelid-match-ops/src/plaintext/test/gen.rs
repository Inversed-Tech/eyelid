//! Iris test data generation.

use rand::Rng;

use crate::{iris::conf::IrisConf, plaintext::rotate};

/// Returns a list of common codes. Random codes are only listed once.
/// The random data is different each time the function is called.
pub fn codes<C: IrisConf>() -> Vec<(&'static str, C::IrisCode)> {
    vec![
        ("set", set_iris_code::<C>()),
        ("unset", unset_iris_code::<C>()),
        ("random", random_iris_code::<C>()),
    ]
}

/// Returns a list of common masks. Random masks are only listed once.
/// The random data is different each time the function is called.
pub fn masks<C: IrisConf>() -> Vec<(&'static str, C::IrisMask)> {
    vec![
        ("visible", visible_iris_mask::<C>()),
        ("occluded", occluded_iris_mask::<C>()),
        ("random", random_iris_mask::<C>()),
    ]
}

/// Returns an iris code with uniformly random bits.
pub fn random_iris_code<C: IrisConf>() -> C::IrisCode {
    let mut code = C::IrisCode::ZERO;
    let mut rng = rand::thread_rng();

    rng.fill(code.data.as_mut_slice());

    code
}

/// Returns an iris code that is similar to the given code.
pub fn similar_iris_code<C: IrisConf>(base: &C::IrisCode) -> C::IrisCode {
    let mut similar = *base;
    // Flip a third of the bits.
    for i in 0..base.len() / 3 {
        let mut b = similar.get_mut(i * 3).expect("bit should exist");
        *b ^= true;
    }
    similar
}

/// Rotate the given iris code within tolerance, such that it should still match.
#[allow(clippy::cast_possible_wrap)]
pub fn rotate_not_too_much<C: IrisConf>(base: &C::IrisCode) -> C::IrisCode {
    rotate::<C>(*base, C::ROTATION_LIMIT as isize)
}

/// Rotate the given iris code so much that it should not match.
#[allow(clippy::cast_possible_wrap)]
pub fn rotate_too_much<C: IrisConf>(base: &C::IrisCode) -> C::IrisCode {
    rotate::<C>(*base, C::ROTATION_LIMIT as isize + 1)
}

/// Returns an iris mask with uniformly random bits.
pub fn random_iris_mask<C: IrisConf>() -> C::IrisMask {
    let mut code = C::IrisMask::ZERO;
    let mut rng = rand::thread_rng();

    rng.fill(code.data.as_mut_slice());

    code
}

/// Returns an iris code with all bits set.
pub fn set_iris_code<C: IrisConf>() -> C::IrisCode {
    !C::IrisCode::ZERO
}

/// Returns an iris code with no bits set.
pub fn unset_iris_code<C: IrisConf>() -> C::IrisCode {
    C::IrisCode::ZERO
}

/// Returns an iris mask that is fully visible (no occlusions).
pub fn visible_iris_mask<C: IrisConf>() -> C::IrisMask {
    !C::IrisCode::ZERO
}

/// Returns an iris mask that is totally occluded.
pub fn occluded_iris_mask<C: IrisConf>() -> C::IrisMask {
    C::IrisCode::ZERO
}
