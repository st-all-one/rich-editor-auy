//! Decoder options.

/// Decoder options.
#[derive(Clone, Copy, Debug, Default)]
pub struct ParseOptions {
    /// Hard limits applied while building the document.
    pub limits: crate::Limits,
}
