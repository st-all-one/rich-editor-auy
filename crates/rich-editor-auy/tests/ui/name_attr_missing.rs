use rich_editor_auy_macros::schema;

fn main() {
    let _ = schema! {
        doc { content: "block+" }
        widget { inline: true, atom: true, a11y: { role: image, name: attr("missing") } }
    };
}
