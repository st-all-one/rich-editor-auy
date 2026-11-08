use std::time::Instant;
use taino_edit_core::{EditorState, Fragment, NodeSpec, SchemaBuilder, Slice};
use taino_edit_extensions::{
    Blockquote, Bold, Code, CodeBlock, Heading, History, Image, Italic, Link, Lists, Paragraph,
    Table, build_schema_with,
};

fn main() {
    let base = SchemaBuilder::new()
        .node(
            "doc",
            NodeSpec {
                content: Some("block+".into()),
                ..Default::default()
            },
        )
        .node(
            "text",
            NodeSpec {
                group: Some("inline".into()),
                ..Default::default()
            },
        );
    let exts: Vec<&dyn taino_edit_extensions::Extension> = vec![
        &Paragraph,
        &Heading,
        &Bold,
        &Italic,
        &Link,
        &Code,
        &Lists,
        &Blockquote,
        &CodeBlock,
        &Image,
        &Table,
        &History,
    ];
    let schema = build_schema_with(base, &exts, "doc").expect("schema");
    let input = std::fs::read_to_string("/tmp/complex-taino.json").expect("input");
    let value: serde_json::Value = serde_json::from_str(&input).expect("json");
    let doc = schema.node_from_json(&value).expect("taino parse");
    let json = serde_json::to_string(&doc.to_json()).expect("taino serialize");
    println!("taino blocks={} json={}", doc.child_count(), json.len());
    let time = |label: &str, mut f: Box<dyn FnMut()>| {
        for _ in 0..5 {
            f();
        }
        let t = Instant::now();
        for _ in 0..50 {
            f();
        }
        println!(
            "{label}: {:.3} ms",
            t.elapsed().as_secs_f64() * 1000.0 / 50.0
        );
    };
    time(
        "taino parse",
        Box::new(|| {
            let _ = schema.node_from_json(&value).expect("parse");
        }),
    );
    time(
        "taino serialize",
        Box::new(|| {
            let _ = serde_json::to_string(&doc.to_json()).expect("ser");
        }),
    );
    let state = EditorState::new(doc.clone(), schema.clone());
    time(
        "taino apply insert",
        Box::new(|| {
            let mut tx = state.tr();
            let text = schema.text("x", vec![]).expect("text");
            let slice = Slice::new(Fragment::from_node(text), 0, 0);
            tx.transform().insert(2, slice, &schema).expect("insert");
            let _ = state.apply(tx);
        }),
    );
}
