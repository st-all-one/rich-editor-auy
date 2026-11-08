//! Checked arithmetic and saturating conversions used by the data path.

/// Adds two values, returning `None` on overflow.
#[must_use]
pub const fn checked_add(left: u32, right: u32) -> Option<u32> {
    left.checked_add(right)
}

/// Subtracts two values, returning `None` on underflow.
#[must_use]
pub const fn checked_sub(left: u32, right: u32) -> Option<u32> {
    left.checked_sub(right)
}

/// Converts a `usize` count to `u32`, rejecting values that do not fit.
#[must_use]
pub fn usize_to_u32(value: usize) -> Option<u32> {
    u32::try_from(value).ok()
}

/// Converts a `u32` to `usize` without a lossy cast on supported targets.
#[must_use]
pub fn u32_to_usize(value: u32) -> Option<usize> {
    usize::try_from(value).ok()
}

/// Converts a `u64` to `u32`, rejecting values that do not fit.
#[must_use]
pub fn u64_to_u32(value: u64) -> Option<u32> {
    u32::try_from(value).ok()
}
