//! Decodes fragments and slices, then checks canonical byte round-trips.
#![no_main]

use libfuzzer_sys::fuzz_target;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{ParseOptions, from_fragment_json, from_slice_json};

fuzz_target!(|data: &[u8]| {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };
    let options = ParseOptions::default();
    if let Ok(fragment) = from_fragment_json(text, &schema, &options) {
        let canonical = fragment.to_json_string(&schema);
        if let Ok(again) = from_fragment_json(&canonical, &schema, &options) {
            assert_eq!(canonical, again.to_json_string(&schema));
        }
    }
    if let Ok(slice) = from_slice_json(text, &schema, &options) {
        let canonical = slice.to_json_string(&schema);
        if let Ok(again) = from_slice_json(&canonical, &schema, &options) {
            assert_eq!(canonical, again.to_json_string(&schema));
        }
    }
});
