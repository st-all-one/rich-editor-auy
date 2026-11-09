use rich_editor_auy_macros::schema;

fn main() {
    let _ = schema! {
        doc { content: "block+" }
        paragraph { group: "block", content: "text*", bogus: true }
    };
}
