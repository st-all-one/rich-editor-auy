#![no_main]

use libfuzzer_sys::fuzz_target;
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_json};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Ok(doc) = from_json(text, &schema, &ParseOptions::default()) else {
        return;
    };
    let wire = doc.to_json_string(&schema);
    assert!(!wire.is_empty(), "canonical writer must not fail on a parsed document");
    let Ok(again) = from_json(&wire, &schema, &ParseOptions::default()) else {
        panic!("canonical output must parse: {wire}");
    };
    assert_eq!(again.to_json_string(&schema), wire, "canonical bytes are stable");
    let tree = again.a11y_tree(&schema);
    let _ = tree.to_json_string(&schema);
    let _ = again.validate_a11y(&schema);
    let size = again.size().0;
    let slice = again.slice(Range::new(Pos(0), Pos(size)));
    assert!(slice.is_ok(), "the whole document is always copyable");
    if let Ok(slice) = slice {
        assert_eq!(slice.size(), size, "a full copy keeps the document size");
        let json = slice.to_json_string(&schema);
        let decoded = rich_editor_auy::wire::from_slice_json(&json, &schema, &ParseOptions::default());
        assert!(decoded.is_ok(), "slice wire round-trips: {json}");
    }
});
