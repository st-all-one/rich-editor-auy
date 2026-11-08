#![no_main]

use libfuzzer_sys::fuzz_target;
use rich_editor_auy::pos::{Pos, Range};
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::EditorState;
use rich_editor_auy::wire::{ParseOptions, from_json};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };
    let Ok(schema) = Schema::basic() else {
        return;
    };
    let Ok(doc) = from_json(text, &schema, &ParseOptions::default()) else {
        return;
    };
    let schema = std::sync::Arc::new(schema);
    let state = EditorState::from_doc(schema, doc);
    let mut transaction = state.transaction();
    let end = state.doc().size().0.saturating_sub(1);
    if end > 0 {
        let _ = transaction.delete(Range::new(Pos(1), Pos(end)));
        let _ = transaction.insert_text(Pos(1), "fuzz");
        let _ = state.apply(transaction);
    }
});
