//! Ordered attribute and mark collections.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::ids::{AttrId, RefTarget};

/// A closed canonical attribute value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttrValue {
    /// Explicit null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Signed integer.
    Int(i64),
    /// UTF-8 string.
    Str(Box<str>),
    /// Ordered array.
    Array(Box<[Self]>),
    /// Reference target, the only nested object in the wire format.
    Ref(RefTarget),
}

/// One ordered attribute pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attr {
    /// Attribute declaration index.
    pub id: AttrId,
    /// Canonical value.
    pub value: AttrValue,
}

/// Attributes sorted by declaration index, without duplicates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Attrs(Box<[Attr]>);

impl<'a> IntoIterator for &'a Attrs {
    type Item = &'a Attr;
    type IntoIter = core::slice::Iter<'a, Attr>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Attrs {
    /// Creates an empty collection.
    #[must_use]
    pub fn empty() -> Self {
        Self(Box::new([]))
    }

    /// Sorts and validates a collection.
    ///
    /// # Errors
    ///
    /// Returns `None` when two attributes share an identifier.
    #[must_use]
    pub fn new(mut values: Vec<Attr>) -> Option<Self> {
        values.sort_by_key(|attr| attr.id);
        if values.windows(2).any(|pair| match pair {
            [left, right] => left.id == right.id,
            _ => false,
        }) {
            return None;
        }
        Some(Self(values.into_boxed_slice()))
    }

    /// Returns an attribute by declaration index.
    #[must_use]
    pub fn get(&self, id: AttrId) -> Option<&AttrValue> {
        self.0
            .binary_search_by_key(&id, |attr| attr.id)
            .ok()
            .and_then(|index| self.0.get(index))
            .map(|attr| &attr.value)
    }

    /// Iterates attributes in declaration order.
    pub fn iter(&self) -> core::slice::Iter<'_, Attr> {
        self.0.iter()
    }

    /// Returns the number of attributes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether no attributes are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a copy with one attribute replaced or inserted.
    #[must_use]
    pub fn with(&self, attr: Attr) -> Option<Self> {
        let mut values = self.0.to_vec();
        match values.binary_search_by_key(&attr.id, |item| item.id) {
            Ok(index) => {
                let slot = values.get_mut(index)?;
                *slot = attr;
            }
            Err(index) => values.insert(index, attr),
        }
        Self::new(values)
    }

    /// Returns a copy without one attribute.
    #[must_use]
    pub fn without(&self, id: AttrId) -> Self {
        Self(
            self.0
                .iter()
                .filter(|attr| attr.id != id)
                .cloned()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }

    /// Returns the underlying slice.
    #[must_use]
    pub fn as_slice(&self) -> &[Attr] {
        &self.0
    }
}
