#![no_main]

use libfuzzer_sys::fuzz_target;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_json};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };
    let Ok(schema) = Schema::full() else {
        return;
    };
    if let Ok(doc) = from_json(text, &schema, &ParseOptions::default()) {
        let tree = doc.a11y_tree(&schema);
        let _ = tree.to_json_string(&schema);
        let _ = doc.validate_a11y(&schema);
    }
});
