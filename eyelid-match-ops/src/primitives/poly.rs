//! Cyclotomic polynomials using ark-poly.

//! This module file is import-only, code is in submodules:
//! - [`Poly`] is in [`modular_poly`] and its submodules,
//! - [`Coeff`] is in [`fq`] and submodules.

pub use fq::Coeff;
pub use modular_poly::{
    modulus::{
        mod_poly, mod_poly_ark_ref_slow, new_unreduced_poly_modulus_slow, FULL_RES_POLY_DEGREE,
    },
    mul::mul_poly,
    Poly,
};

// Use `mod_poly` outside this module, it is set to the fastest modulus operation.
#[cfg(not(any(test, feature = "benchmark")))]
use modular_poly::modulus::mod_poly_manual_mut;
#[cfg(any(test, feature = "benchmark"))]
pub use modular_poly::modulus::mod_poly_manual_mut;

// Use `mul_poly` outside this module, it is set to the fastest multiplication operation.
#[cfg(not(any(test, feature = "benchmark")))]
use modular_poly::mul::{
    flat_karatsuba_mul, naive_cyclotomic_mul, poly_split, poly_split_half, rec_karatsuba_mul,
};
#[cfg(any(test, feature = "benchmark"))]
pub use modular_poly::mul::{
    flat_karatsuba_mul, naive_cyclotomic_mul, poly_split, poly_split_half, rec_karatsuba_mul,
};

pub mod fq;
pub mod modular_poly;

#[cfg(any(test, feature = "benchmark"))]
pub mod test;

// Do not add code here.
// Add functions or trait impls to modular_poly/*.rs and inherent method impls to modular_poly.rs.
