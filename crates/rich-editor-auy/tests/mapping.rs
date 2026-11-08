//! Position mapping invariants.

use rich_editor_auy::pos::{Assoc, Pos};
use rich_editor_auy::step::StepMap;

#[test]
fn positions_after_a_deletion_shift_back() {
    let map = StepMap::replacement(Pos(1), 2, 0);
    assert_eq!(map.map_pos(Pos(5), Assoc::After), Some(Pos(3)));
}

#[test]
fn positions_after_an_insertion_shift_forward() {
    let map = StepMap::replacement(Pos(1), 0, 2);
    assert_eq!(map.map_pos(Pos(5), Assoc::After), Some(Pos(7)));
}

#[test]
fn positions_before_a_range_are_unchanged() {
    let map = StepMap::replacement(Pos(4), 2, 1);
    assert_eq!(map.map_pos(Pos(1), Assoc::Before), Some(Pos(1)));
}
