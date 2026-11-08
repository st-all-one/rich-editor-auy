//! Byte positions, structural sizes, and ranges.

use core::fmt;

/// A byte offset in the canonical document space.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(pub u32);

impl Pos {
    /// The zero position.
    pub const ZERO: Self = Self(0);

    /// Creates a position.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw offset.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Adds an offset, returning `None` on overflow.
    #[must_use]
    pub const fn checked_add(self, amount: u32) -> Option<Self> {
        match self.0.checked_add(amount) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Subtracts an offset, returning `None` on underflow.
    #[must_use]
    pub const fn checked_sub(self, amount: u32) -> Option<Self> {
        match self.0.checked_sub(amount) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A document size in the same unit as [`Pos`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size(pub u32);

impl Size {
    /// The zero size.
    pub const ZERO: Self = Self(0);

    /// Creates a size.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw size.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Converts the size to a position.
    #[must_use]
    pub const fn as_pos(self) -> Pos {
        Pos(self.0)
    }

    /// Adds a size, returning `None` on overflow.
    #[must_use]
    pub const fn checked_add(self, amount: u32) -> Option<Self> {
        match self.0.checked_add(amount) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A half-open range in document space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Range<T = Pos> {
    /// Inclusive start.
    pub start: T,
    /// Exclusive end.
    pub end: T,
}

impl<T> Range<T> {
    /// Creates a range without validating order.
    #[must_use]
    pub const fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
}

impl Range<Pos> {
    /// Returns the covered byte count.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.end.0.saturating_sub(self.start.0)
    }

    /// Returns whether the range is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start.0 >= self.end.0
    }

    /// Returns whether a position lies inside the half-open range.
    #[must_use]
    pub const fn contains(self, position: Pos) -> bool {
        position.0 >= self.start.0 && position.0 < self.end.0
    }

    /// Returns whether the range is ordered.
    #[must_use]
    pub const fn is_ordered(self) -> bool {
        self.start.0 <= self.end.0
    }
}

impl Range<Pos> {
    /// Clamps the range into `[0, size]`.
    #[must_use]
    pub const fn clamp(self, size: Pos) -> Self {
        let start = if self.start.0 > size.0 {
            size
        } else {
            self.start
        };
        let end = if self.end.0 > size.0 { size } else { self.end };
        Self { start, end }
    }
}

/// Resolved selection association at insertion boundaries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Assoc {
    /// Stay before inserted content.
    Before,
    /// Stay after inserted content.
    #[default]
    After,
}

impl Assoc {
    /// Returns `-1` for before and `1` for after.
    #[must_use]
    pub const fn sign(self) -> i8 {
        match self {
            Self::Before => -1,
            Self::After => 1,
        }
    }
}

/// A structural document path, stored as stable slash-separated indices.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocPath(pub alloc::boxed::Box<str>);

impl DocPath {
    /// Creates an empty path.
    #[must_use]
    pub fn root() -> Self {
        Self(alloc::boxed::Box::from(""))
    }

    /// Appends one index component.
    #[must_use]
    #[allow(
        clippy::format_push_string,
        reason = "diagnostic path construction is cold"
    )]
    pub fn child(&self, index: u32) -> Self {
        let mut value = alloc::string::String::from(&*self.0);
        if !value.is_empty() {
            value.push('/');
        }
        value.push_str(&alloc::format!("{index}"));
        Self(value.into_boxed_str())
    }

    /// Returns the path as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DocPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
