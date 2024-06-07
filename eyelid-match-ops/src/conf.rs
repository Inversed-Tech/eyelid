//! Configuration marker types.
//! Any or all of the configuration traits can be implemented on these types, or your own custom
//! types.

/// Raw full resolution iris code dimensions.
///
/// This uses the full number of iris bits, which gives an upper bound on benchmarks.
///
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FullBits;

/// Raw middle resolution iris code dimensions.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MiddleBits;

/// Tiny test polynomials, used for finding edge cases in tests.
/// Used for both a tiny resolution and a tiny block encoding.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.
#[cfg(tiny_poly)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TinyTest;

/// The polynomial config used in tests.
//
// We use the full resolution by default, but TinyTest when cfg(tiny_poly) is set.
#[cfg(not(tiny_poly))]
pub type TestBits = FullBits;

/// The polynomial config used in tests.
///
/// Temporarily switch to this tiny field to make test errors easier to debug:
/// ```no_run
/// RUSTFLAGS="--cfg tiny_poly" cargo test
/// RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
/// ```
#[cfg(tiny_poly)]
pub type TestBits = TinyTest;
