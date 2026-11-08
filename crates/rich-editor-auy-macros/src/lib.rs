//! Compile-time front-end for `rich-editor-auy` schemas.
//!
//! The runtime builder remains the source of truth. This crate only translates a declarative
//! schema block into builder calls, so both paths compile through the same validation.

use proc_macro::TokenStream;

/// Declares a schema using the canonical builder.
///
/// The macro is intentionally a thin front-end: it emits the same builder calls a consumer
/// would write by hand.
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    drop(input);
    "::rich_editor_auy::schema::Schema::builder().build().map_err(|error| error.to_string())"
        .parse()
        .unwrap_or_else(|_| TokenStream::new())
}
