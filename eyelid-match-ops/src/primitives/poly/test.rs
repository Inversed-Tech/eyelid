//! Tests for basic polynomial operations.

#[cfg(any(test, feature = "benchmark"))]
pub mod gen;

#[cfg(test)]
pub mod mul;

#[cfg(any(test, feature = "benchmark"))]
pub mod inv;
