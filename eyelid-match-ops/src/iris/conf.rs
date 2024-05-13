//! Iris configurations for encoding and encrption schemes.

/// The dimensions and matching rules for an iris code.
pub trait IrisConf {
    /// The number of rows in an iris code or iris mask.
    const COLUMN_LENGTH: usize;

    /// The number of columns in an iris code or iris mask.
    const COLUMNS: usize;

    /// The length of an iris code or iris mask.
    const BIT_LENGTH: usize = Self::COLUMN_LENGTH * Self::COLUMNS;

    /// The rotation limit when comparing irises.
    /// Each column is compared to the [`ROTATION_LIMIT`](Self::ROTATION_LIMIT) columns to its left and right.
    const ROTATION_LIMIT: usize;

    /// The number of rotations used when comparing irises.
    /// This includes the comparison with no rotation.
    const ROTATION_COMPARISONS: usize = Self::ROTATION_LIMIT * 2 + 1;

    /// The numerator of the bit match threshold for a successful iris match.
    /// The default match threshold is 36%.
    const MATCH_NUMERATOR: usize = 36;

    /// The denominator of the bit match threshold for a successful iris match.
    /// The default match threshold is 36%.
    const MATCH_DENOMINATOR: usize = 100;
}
