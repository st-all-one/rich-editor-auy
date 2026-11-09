use rich_editor_auy_macros::schema;

fn main() {
    let _ = schema! {
        doc { content: "block+" }
        paragraph { group: "block", content: "text*", a11y: { role: widget } }
    };
}
