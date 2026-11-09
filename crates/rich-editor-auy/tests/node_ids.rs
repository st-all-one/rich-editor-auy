//! Deterministic anchor derivation.

use rich_editor_auy::model::NodeId;

#[test]
fn derived_ids_are_stable_and_path_sensitive() {
    let first = NodeId::derived("seed", &[0]);
    let again = NodeId::derived("seed", &[0]);
    let second = NodeId::derived("seed", &[1]);
    let other_seed = NodeId::derived("other", &[0]);
    assert!(first.is_some(), "seed and path derive an id");
    let (Some(first), Some(again), Some(second), Some(other_seed)) =
        (first, again, second, other_seed)
    else {
        return;
    };
    assert_eq!(first, again, "same seed and path are stable");
    assert_ne!(first, second, "path changes the id");
    assert_ne!(first, other_seed, "seed changes the id");
    assert_eq!(first.as_str().len(), 64, "ids are 64 hex characters");
    assert!(
        first.as_str().chars().all(|c| c.is_ascii_hexdigit()),
        "ids are hexadecimal"
    );
}
