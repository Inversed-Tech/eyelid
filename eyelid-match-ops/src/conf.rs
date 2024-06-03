//! Configuration marker types.
//! Any or all of the configuration traits can be implemented on these types, or your own custom
//! types.

/// The polynomial config used in tests.
//
// We use the full resolution by default, but TinyTest when cfg(tiny_poly) is set.
#[cfg(not(tiny_poly))]
pub type TestRes = FullRes;

/// The polynomial config used in tests.
///
/// Temporarily switch to this tiny field to make test errors easier to debug:
/// ```no_run
/// RUSTFLAGS="--cfg tiny_poly" cargo test
/// RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
/// ```
#[cfg(tiny_poly)]
pub type TestRes = TinyTest;

/// Iris bit length polynomial parameters.
///
/// This uses the full number of iris bits, which gives an upper bound on benchmarks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IrisBits;

/// Full resolution polynomial parameters.
///
/// These are the parameters for full resolution, according to the Inversed Tech report.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FullRes;

/// Middle resolution polynomial parameters.
///
/// These are the parameters for middle resolution, according to the Inversed Tech report.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MiddleRes;

/// Tiny test polynomials, used for finding edge cases in tests.
///
/// The test parameters are specifically chosen to make failing tests easy to read and diagnose.
#[cfg(tiny_poly)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TinyTest;
