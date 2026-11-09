use rich_editor_auy_macros::schema;

fn main() {
    let _ = schema! {
        doc { content: "block+" }
        heading {
            group: "block",
            content: "text*",
            attrs: { level: int(1..=6) = 1 },
            a11y: { role: heading(2), name: text }
        }
    };
}
