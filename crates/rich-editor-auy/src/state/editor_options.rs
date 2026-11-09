//! Editor construction options.

use alloc::sync::Arc;

use crate::error::SchemaError;
use crate::model::Doc;
use crate::schema::Schema;

use super::{Editor, EditorOptions, EditorState};

impl Editor {
    /// Creates an empty editor with explicit construction options.
    ///
    /// # Errors
    ///
    /// Returns a schema error only if the full preset is inconsistent.
    pub fn with_options(options: EditorOptions) -> Result<Self, SchemaError> {
        let schema = Arc::new(Schema::full()?);
        let state = EditorState::new(schema)?;
        let state = match options.history_limits {
            Some((max_groups, max_bytes)) => state.with_history_limits(max_groups, max_bytes),
            None => state,
        };
        Ok(Self { state, version: 0 })
    }

    /// Creates an editor around an existing document.
    ///
    /// The document must already conform to the complete schema; this constructor does not
    /// re-validate it.
    ///
    /// # Errors
    ///
    /// Returns a schema error only if the full preset is inconsistent.
    pub fn from_doc(doc: Doc) -> Result<Self, SchemaError> {
        let schema = Arc::new(Schema::full()?);
        Ok(Self {
            state: EditorState::from_doc(schema, doc),
            version: 0,
        })
    }

    /// Creates an editor around an existing document with explicit options.
    ///
    /// The document must already conform to the complete schema; this constructor does not
    /// re-validate it.
    ///
    /// # Errors
    ///
    /// Returns a schema error only if the full preset is inconsistent.
    pub fn from_doc_with_options(doc: Doc, options: EditorOptions) -> Result<Self, SchemaError> {
        let schema = Arc::new(Schema::full()?);
        let state = EditorState::from_doc(schema, doc);
        let state = match options.history_limits {
            Some((max_groups, max_bytes)) => state.with_history_limits(max_groups, max_bytes),
            None => state,
        };
        Ok(Self { state, version: 0 })
    }
}
