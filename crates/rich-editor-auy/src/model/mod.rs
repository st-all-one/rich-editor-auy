//! Canonical document model.

pub mod attrs;
pub mod block;
pub(crate) mod block_seq;
mod chunk;
mod chunks;
pub mod doc;
pub mod ids;
mod limits;
pub mod marks;
mod measure;
mod records;
pub mod text;
pub mod validate;

pub use attrs::{Attr, AttrValue, Attrs};
pub use block::{AtomData, Block, BlockContent, BlockIter, InlineIter, InlineView};
pub use block_seq::{BlockSeq, TopBlockIter};
pub use doc::{Doc, Fragment};
pub use ids::{AttrId, DocId, GroupId, MarkId, NodeId, NodeKind, RefTarget, VersionTag};
pub use marks::{Mark, MarkAttr, MarkSet};
pub use records::{Anchor, Reference};
pub use text::{InlineAtom, InlineItem, MAX_RUN_BYTES, Run, TextBlock};
pub use validate::{DocIssue, DocReport};
