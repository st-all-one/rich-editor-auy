use rich_editor_auy_macros::schema;

fn main() {
    const DEFAULT: i64 = 1;

    let _ = schema! {
        doc { content: "block+" }
        paragraph { group: "block", content: "text*", attrs: { level: int(1..=6) = DEFAULT } }
    };
}
