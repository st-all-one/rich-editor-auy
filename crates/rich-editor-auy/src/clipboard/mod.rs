//! Copy, paste, and range deletion.
//!
//! A [`crate::step::Slice`] is the payload of a clipboard operation: inline runs with marks, or a detached
//! block sequence. The module owns three rules that the rest of the crate relies on:
//!
//! 1. copied blocks never carry a block anchor (`id`), because anchors identify nodes inside one
//!    document;
//! 2. a multi-block range starts and ends on block boundaries, except for the two partial text
//!    blocks at the edges;
//! 3. a delete always leaves an editable text block where the caret lands.

mod delete;
mod insert;
mod open;
mod selection;
