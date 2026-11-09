//! Transaction constructors for document, inline, and mark attribute edits.

use alloc::boxed::Box;

use crate::error::StepError;
use crate::model::{AttrId, AttrValue, Doc, MarkId};
use crate::pos::Range;
use crate::step::Step;

use super::transaction::Transaction;

#[allow(
    clippy::unnecessary_wraps,
    reason = "constructors share a fallible step API"
)]
impl Transaction {
    /// Replaces the whole document with a snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error when the transaction has no base document to measure.
    pub fn replace_document(&mut self, doc: Doc) -> Result<(), StepError> {
        let Some(base) = self.base.as_ref() else {
            return Err(StepError::Unsupported {
                operation: Box::from("replace_document_without_base"),
            });
        };
        let old_size = base.size().0;
        self.steps.push(Step::ReplaceDoc { doc, old_size });
        Ok(())
    }

    /// Sets an attribute on one inline atom.
    ///
    /// # Errors
    ///
    /// This constructor is infallible; validation happens in `apply`.
    pub fn set_inline_attr(
        &mut self,
        range: Range,
        attr: AttrId,
        value: AttrValue,
    ) -> Result<(), StepError> {
        self.steps.push(Step::SetInlineAttr {
            from: range.start,
            to: range.end,
            attr,
            value,
        });
        Ok(())
    }

    /// Sets an attribute on every run carrying a mark inside a range.
    ///
    /// An empty range carries no mark and queues no step.
    ///
    /// # Errors
    ///
    /// This constructor is infallible; validation happens in `apply`.
    pub fn set_mark_attr(
        &mut self,
        range: Range,
        mark: MarkId,
        attr: AttrId,
        value: AttrValue,
    ) -> Result<(), StepError> {
        if range.is_empty() {
            return Ok(());
        }
        self.steps.push(Step::SetMarkAttr {
            from: range.start,
            to: range.end,
            mark,
            attr,
            value,
        });
        Ok(())
    }
}
