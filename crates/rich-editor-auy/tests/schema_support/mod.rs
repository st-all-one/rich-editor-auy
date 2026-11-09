//! Shared helpers for full-schema tests.

use rich_editor_auy::schema::Schema;

#[allow(unreachable_pub, reason = "shared by several integration test crates")]
pub fn full() -> Option<Schema> {
    let schema = Schema::full();
    assert!(schema.is_ok(), "full schema must compile");
    schema.ok()
}
