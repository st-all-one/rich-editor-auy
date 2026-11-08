//! IME composition values.

use crate::pos::Range;

/// One IME composition identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositionId(pub u64);

/// Composition events supplied by the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompositionEvent {
    /// Start at a range.
    Start {
        /// Replaced range.
        at: Range,
    },
    /// Update the composed text.
    Update {
        /// Current composed text.
        text: alloc::boxed::Box<str>,
    },
    /// Commit the composition as one undo group.
    Commit,
    /// Cancel and restore the original text.
    Cancel,
}
