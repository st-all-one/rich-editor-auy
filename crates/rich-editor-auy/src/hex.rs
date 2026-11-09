//! Lowercase hexadecimal encoding shared by every digest.
#![expect(
    unreachable_pub,
    reason = "the module is crate-internal, so the helper is not part of the published API"
)]

use alloc::string::String;

/// Encodes bytes as lowercase hexadecimal.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        let high = usize::from(byte >> 4);
        let low = usize::from(byte & 0x0f);
        if let (Some(high), Some(low)) = (DIGITS.get(high), DIGITS.get(low)) {
            out.push(char::from(*high));
            out.push(char::from(*low));
        }
    }
    out
}
