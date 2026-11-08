//! Hostile input limits.

use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_json};

#[test]
fn deep_json_is_rejected_before_recursion_exhausts_the_stack() {
    let schema = Schema::basic();
    assert!(schema.is_ok(), "basic schema must compile");
    let Ok(schema) = schema else {
        return;
    };
    let input = "[".repeat(1_000);
    let parsed = from_json(&input, &schema, &ParseOptions::default());
    assert!(parsed.is_err(), "deep JSON must fail");
    let Err(error) = parsed else {
        return;
    };
    assert_eq!(&*error.code, "depth_limit");
}
