//! Iris code matching operations library.
//!
//! This library has 3 core modules:
//! [`plaintext`]: operations on raw bit vectors,
//! [`encoded`]: the same operations on polynomial-encoded bit vectors,
//! [`encrypted`]: the same operations on fully homomorphic encrypted, polynomial-encoded bit
//!                vectors.

pub mod encoded;
pub mod encrypted;
pub mod plaintext;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
