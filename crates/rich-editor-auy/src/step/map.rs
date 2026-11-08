//! Ordered step maps and position mapping.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit edit algebra and mapping branches"
)]

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::pos::{Assoc, Pos, Range};

/// One replacement range in a step map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepMapRange {
    /// Start in the old document.
    pub start: u32,
    /// Removed byte count.
    pub old_len: u32,
    /// Inserted byte count.
    pub new_len: u32,
}

/// Ordered, non-overlapping ranges for one step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StepMap {
    ranges: Box<[StepMapRange]>,
}

impl StepMap {
    /// Creates an identity map.
    #[must_use]
    pub fn identity() -> Self {
        Self::default()
    }

    /// Creates a map from ordered ranges.
    ///
    /// # Errors
    ///
    /// Returns `None` when ranges overlap or are not sorted.
    #[must_use]
    pub fn new(ranges: Vec<StepMapRange>) -> Option<Self> {
        let mut previous_end = 0_u32;
        for range in &ranges {
            if range.start < previous_end {
                return None;
            }
            previous_end = range.start.saturating_add(range.old_len);
        }
        Some(Self {
            ranges: ranges.into_boxed_slice(),
        })
    }

    /// Creates a map for one replacement.
    #[must_use]
    pub fn replacement(start: Pos, old_len: u32, new_len: u32) -> Self {
        if old_len == new_len {
            return Self::default();
        }
        Self {
            ranges: Box::new([StepMapRange {
                start: start.0,
                old_len,
                new_len,
            }]),
        }
    }

    /// Returns the ranges.
    #[must_use]
    pub fn ranges(&self) -> &[StepMapRange] {
        &self.ranges
    }

    /// Maps a position through the map.
    #[must_use]
    pub fn map_pos(&self, pos: Pos, assoc: Assoc) -> Option<Pos> {
        let mut current = pos.0;
        for range in &self.ranges {
            let old_end = range.start.checked_add(range.old_len)?;
            if current < range.start {
                break;
            }
            if current > old_end {
                current = if range.new_len < range.old_len {
                    current.checked_sub(range.old_len.checked_sub(range.new_len)?)?
                } else {
                    current.checked_add(range.new_len.checked_sub(range.old_len)?)?
                };
                continue;
            }
            if current == range.start {
                let mapped = match assoc {
                    Assoc::Before => range.start,
                    Assoc::After => range.start.checked_add(range.new_len)?,
                };
                return Some(Pos(mapped));
            }
            if current == old_end {
                return Some(Pos(range.start.checked_add(range.new_len)?));
            }
            let offset = current.checked_sub(range.start)?;
            let clamped = offset.min(range.new_len);
            return Some(Pos(range.start.checked_add(clamped)?));
        }
        Some(Pos(current))
    }

    /// Maps a range.
    #[must_use]
    pub fn map_range(&self, range: Range, assoc: Assoc) -> Option<Range> {
        Some(Range::new(
            self.map_pos(range.start, assoc)?,
            self.map_pos(range.end, assoc)?,
        ))
    }
}

/// A composition of step maps in application order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mapping {
    maps: Box<[StepMap]>,
}

impl Mapping {
    /// Creates an empty mapping.
    #[must_use]
    pub fn identity() -> Self {
        Self::default()
    }

    /// Appends a map.
    #[must_use]
    pub fn with_map(self, map: StepMap) -> Self {
        let mut maps = self.maps.to_vec();
        maps.push(map);
        Self {
            maps: maps.into_boxed_slice(),
        }
    }

    /// Returns the maps.
    #[must_use]
    pub fn maps(&self) -> &[StepMap] {
        &self.maps
    }

    /// Maps a position through every map.
    #[must_use]
    pub fn map_pos(&self, mut pos: Pos, assoc: Assoc) -> Option<Pos> {
        for map in &self.maps {
            pos = map.map_pos(pos, assoc)?;
        }
        Some(pos)
    }

    /// Maps a range through every map.
    #[must_use]
    pub fn map_range(&self, mut range: Range, assoc: Assoc) -> Option<Range> {
        for map in &self.maps {
            range = map.map_range(range, assoc)?;
        }
        Some(range)
    }
}
