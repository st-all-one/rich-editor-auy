use rich_editor_auy_macros::schema;

fn main() {
    let _ = schema! {
        doc { content: "block+" }
        link { mark, attrs: { href: str, href: str } }
    };
}
