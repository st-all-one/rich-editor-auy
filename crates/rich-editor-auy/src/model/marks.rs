//! Marks and ordered mark sets.
#![allow(clippy::excessive_nesting, reason = "deterministic mark union")]

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::attrs::Attrs;
use super::ids::MarkId;

/// One mark with its typed attributes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    /// Mark declaration index.
    pub id: MarkId,
    /// Attribute values for this mark.
    pub attrs: Attrs,
}

impl Mark {
    /// Creates a mark without attributes.
    #[must_use]
    pub fn new(id: MarkId) -> Self {
        Self {
            id,
            attrs: Attrs::empty(),
        }
    }
}

/// Attributes attached to a mark present in a [`MarkSet`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkAttr {
    /// Mark declaration index.
    pub id: MarkId,
    /// Mark attributes.
    pub attrs: Attrs,
}

/// Bit mask plus ordered attributes for the marks on a run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MarkSet {
    mask: u64,
    attrs: Box<[MarkAttr]>,
}

impl MarkSet {
    /// Creates an empty set.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mask: 0,
            attrs: Box::new([]),
        }
    }

    /// Returns the raw mask.
    #[must_use]
    pub const fn mask(&self) -> u64 {
        self.mask
    }

    /// Returns whether the set has no marks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }

    /// Returns whether a mark is present.
    #[must_use]
    pub fn contains(&self, id: MarkId) -> bool {
        match bit(id) {
            Some(flag) => self.mask & flag != 0,
            None => false,
        }
    }

    /// Returns typed attributes for a mark, when present.
    #[must_use]
    pub fn attrs_for(&self, id: MarkId) -> Option<&Attrs> {
        self.attrs
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| &entry.attrs)
    }

    /// Iterates marks with attributes in declaration order.
    pub fn iter_attrs(&self) -> core::slice::Iter<'_, MarkAttr> {
        self.attrs.iter()
    }

    /// Returns a set containing exactly one mark.
    #[must_use]
    pub fn single(mark: Mark) -> Self {
        let attrs = if mark.attrs.is_empty() {
            Vec::new().into_boxed_slice()
        } else {
            Vec::from([MarkAttr {
                id: mark.id,
                attrs: mark.attrs,
            }])
            .into_boxed_slice()
        };
        Self {
            mask: bit(mark.id).unwrap_or(0),
            attrs,
        }
    }

    /// Returns a copy with one mark added or replaced.
    #[must_use]
    pub fn with_mark(&self, mark: Mark) -> Self {
        let Some(flag) = bit(mark.id) else {
            return self.clone();
        };
        let mut attrs = self
            .attrs
            .iter()
            .filter(|entry| entry.id != mark.id)
            .cloned()
            .collect::<Vec<_>>();
        if !mark.attrs.is_empty() {
            attrs.push(MarkAttr {
                id: mark.id,
                attrs: mark.attrs,
            });
        }
        attrs.sort_by_key(|entry| entry.id);
        Self {
            mask: self.mask | flag,
            attrs: attrs.into_boxed_slice(),
        }
    }

    /// Builds a set from marks in one pass.
    ///
    /// Attribute entries are sorted by declaration index. The caller guarantees unique mark
    /// ids; duplicates only repeat mask bits.
    #[must_use]
    pub(crate) fn from_marks(marks: Vec<Mark>) -> Self {
        let mut mask = 0_u64;
        let mut attrs = Vec::with_capacity(marks.len());
        for mark in marks {
            mask |= bit(mark.id).unwrap_or(0);
            if !mark.attrs.is_empty() {
                attrs.push(MarkAttr {
                    id: mark.id,
                    attrs: mark.attrs,
                });
            }
        }
        attrs.sort_by_key(|entry| entry.id);
        Self {
            mask,
            attrs: attrs.into_boxed_slice(),
        }
    }

    /// Returns a copy with one mark removed.
    #[must_use]
    pub fn without_mark(&self, id: MarkId) -> Self {
        let flag = bit(id).unwrap_or(0);
        let attrs = self
            .attrs
            .iter()
            .filter(|entry| entry.id != id)
            .cloned()
            .collect::<Vec<_>>();
        Self {
            mask: self.mask & !flag,
            attrs: attrs.into_boxed_slice(),
        }
    }

    /// Returns whether every mark in `other` is present here.
    #[must_use]
    pub fn is_superset(&self, other: &Self) -> bool {
        self.mask & other.mask == other.mask
    }

    /// Returns the union of two sets, preferring attributes from `self`.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        let mut attrs = self.attrs.to_vec();
        for entry in &other.attrs {
            if !attrs.iter().any(|current| current.id == entry.id) {
                attrs.push(entry.clone());
            }
        }
        attrs.sort_by_key(|entry| entry.id);
        Self {
            mask: self.mask | other.mask,
            attrs: attrs.into_boxed_slice(),
        }
    }
}

fn bit(id: MarkId) -> Option<u64> {
    let shift = u32::from(id.0);
    1_u64.checked_shl(shift)
}
