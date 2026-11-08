//! Canonical document model.

pub mod attrs;
pub mod block;
pub mod doc;
pub mod ids;
pub mod marks;
mod measure;
pub mod text;
pub mod validate;

pub use attrs::{Attr, AttrValue, Attrs};
pub use block::{AtomData, Block, BlockContent, BlockIter, InlineIter, InlineView};
pub use doc::{Anchor, Doc, Fragment, Reference};
pub use ids::{AttrId, DocId, GroupId, MarkId, NodeId, NodeKind, RefTarget, VersionTag};
pub use marks::{Mark, MarkAttr, MarkSet};
pub use text::{InlineAtom, InlineItem, MAX_RUN_BYTES, Run, TextBlock};
pub use validate::{DocIssue, DocReport};
