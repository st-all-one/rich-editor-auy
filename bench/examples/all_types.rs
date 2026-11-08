#![allow(clippy::disallowed_methods, clippy::unit_arg, reason = "benchmark harness")]

//! All-types benchmark document.
//!
//! Run with: cargo run --release --manifest-path bench/Cargo.toml --example all_types

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use rich_editor_auy::a11y::{A11yRole, A11ySpec, AltPolicy, NamePolicy};
use rich_editor_auy::model::{
    Attr, AttrId, AttrValue, Attrs, AtomData, Block, Doc, DocId, InlineAtom, InlineItem, Mark,
    MarkSet, NodeId, RefTarget, Run, TextBlock,
};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::{
    AttrSpec, AttrType, DocSpec, MarkSpec, NodeSpec, Schema,
};
use rich_editor_auy::state::EditorState;
use rich_editor_auy::wire::{ParseOptions, check_json, from_json};

fn attr(id: u16, value: AttrValue) -> Attr {
    Attr { id: AttrId(id), value }
}

fn attrs(values: Vec<Attr>) -> Attrs {
    Attrs::new(values).expect("unique attrs")
}

fn rich_schema() -> Schema {
    let level = AttrSpec::new("level", AttrType::Int { min: 1, max: 6 })
        .required()
        .with_default(AttrValue::Int(2));
    let alt = AttrSpec::new("alt", AttrType::Str).required();
    let src = AttrSpec::new("src", AttrType::Str);
    let doc = DocSpec::new()
        .content("block+")
        .attr(AttrSpec::new("lang", AttrType::Str))
        .attr(AttrSpec::new("dir", AttrType::Str))
        .attr(AttrSpec::new("title", AttrType::Str));
    Schema::builder()
        .doc(doc)
        .node(
            NodeSpec::new("paragraph")
                .content("(text | image)*")
                .group("block")
                .marks_all()
                .a11y(A11ySpec::new(A11yRole::Paragraph).with_name(NamePolicy::TextContent)),
        )
        .node(
            NodeSpec::new("heading")
                .content("(text | image)*")
                .group("block")
                .marks_all()
                .attr(level)
                .a11y(
                    A11ySpec::new(A11yRole::Heading { level: 2 })
                        .with_name(NamePolicy::TextContent)
                        .with_level(AttrId(1)),
                ),
        )
        .node(
            NodeSpec::new("bullet_list")
                .content("list_item+")
                .group("block")
                .a11y(A11ySpec::new(A11yRole::List)),
        )
        .node(
            NodeSpec::new("list_item")
                .content("block+")
                .a11y(A11ySpec::new(A11yRole::ListItem)),
        )
        .node(
            NodeSpec::new("blockquote")
                .content("block+")
                .group("block")
                .a11y(A11ySpec::new(A11yRole::Quote)),
        )
        .node(
            NodeSpec::new("code_block")
                .content("text*")
                .group("block")
                .a11y(A11ySpec::new(A11yRole::CodeBlock)),
        )
        .node(
            NodeSpec::new("table")
                .content("table_row+")
                .group("block")
                .a11y(A11ySpec::new(A11yRole::Table)),
        )
        .node(
            NodeSpec::new("table_row")
                .content("(table_cell | table_header)+")
                .a11y(A11ySpec::new(A11yRole::TableRow)),
        )
        .node(
            NodeSpec::new("table_cell")
                .content("block+")
                .a11y(A11ySpec::new(A11yRole::TableCell { header: false })),
        )
        .node(
            NodeSpec::new("table_header")
                .content("block+")
                .a11y(A11ySpec::new(A11yRole::TableCell { header: true })),
        )
        .node(
            NodeSpec::new("separator")
                .group("block")
                .atom()
                .a11y(A11ySpec::new(A11yRole::Separator)),
        )
        .node(
            NodeSpec::new("image")
                .inline()
                .atom()
                .attr(alt)
                .attr(src)
                .a11y(
                    A11ySpec::new(A11yRole::Image)
                        .with_name(NamePolicy::Attr(AttrId(1)))
                        .with_alt(AltPolicy::Required),
                ),
        )
        .node(NodeSpec::new("text").inline().a11y(A11ySpec::new(A11yRole::Text)))
        .mark(MarkSpec::new("strong"))
        .mark(MarkSpec::new("em"))
        .mark(MarkSpec::new("code"))
        .mark(
            MarkSpec::new("link")
                .attr(AttrSpec::new("href", AttrType::Str).required())
                .a11y(A11ySpec::new(A11yRole::Link).with_name(NamePolicy::TextContent)),
        )
        .mark(MarkSpec::new("ref").attr(AttrSpec::new("target", AttrType::Ref).required()))
        .build()
        .expect("schema")
}

fn run(text: &str, marks: MarkSet) -> InlineItem {
    InlineItem::Text(Run::new(String::from(text), marks))
}

fn make_text(schema: &Schema) -> TextBlock {
    let strong = schema.mark_id("strong").expect("strong");
    let em = schema.mark_id("em").expect("em");
    let code = schema.mark_id("code").expect("code");
    let link = schema.mark_id("link").expect("link");
    let reference = schema.mark_id("ref").expect("ref");
    let image = schema.node_kind("image").expect("image");
    let link_mark = Mark {
        id: link,
        attrs: attrs(vec![attr(0, AttrValue::Str(Box::from("https://example.test/a")))]),
    };
    let ref_mark = Mark {
        id: reference,
        attrs: attrs(vec![attr(
            0,
            AttrValue::Ref(RefTarget {
                doc: DocId::new("pub-2").expect("doc id"),
                node: NodeId::new("node-7"),
                version: None,
                fragment: None,
            }),
        )]),
    };
    let mut forte = MarkSet::single(Mark::new(strong));
    forte = forte.with_mark(Mark::new(em));
    TextBlock::new(vec![
        run("Texto ", MarkSet::empty()),
        run("forte", forte),
        run(" e ", MarkSet::empty()),
        run("ênfase", MarkSet::single(Mark::new(em))),
        run(" com ", MarkSet::empty()),
        run("código", MarkSet::single(Mark::new(code))),
        run(" e ", MarkSet::empty()),
        run("link", MarkSet::single(link_mark)),
        run(". ", MarkSet::empty()),
        InlineItem::Atom(InlineAtom {
            kind: image,
            attrs: attrs(vec![
                attr(1, AttrValue::Str(Box::from("diagrama"))),
                attr(2, AttrValue::Str(Box::from("https://example.test/d.png"))),
            ]),
        }),
        run(" ", MarkSet::empty()),
        run("referência", MarkSet::single(ref_mark)),
        run(".", MarkSet::empty()),
    ])
}

fn paragraph(schema: &Schema, text: TextBlock, id: Option<&str>) -> Arc<Block> {
    let kind = schema.node_kind("paragraph").expect("paragraph");
    let attrs = id.map_or_else(Attrs::empty, |id| {
        attrs(vec![attr(0, AttrValue::Str(Box::from(id)))])
    });
    Arc::new(Block::text(kind, attrs, text))
}

fn make_document(schema: &Schema, repeats: usize) -> Doc {
    let doc_attrs = attrs(vec![
        attr(0, AttrValue::Str(Box::from("pt-BR"))),
        attr(1, AttrValue::Str(Box::from("ltr"))),
        attr(2, AttrValue::Str(Box::from("Corpus complexo"))),
    ]);
    let heading = schema.node_kind("heading").expect("heading");
    let list = schema.node_kind("bullet_list").expect("list");
    let item = schema.node_kind("list_item").expect("item");
    let quote = schema.node_kind("blockquote").expect("quote");
    let code = schema.node_kind("code_block").expect("code");
    let table = schema.node_kind("table").expect("table");
    let row = schema.node_kind("table_row").expect("row");
    let cell = schema.node_kind("table_cell").expect("cell");
    let head = schema.node_kind("table_header").expect("header");
    let separator = schema.node_kind("separator").expect("separator");
    let mut blocks = Vec::new();
    for index in 0..repeats {
        blocks.push(Arc::new(Block::text(
            heading,
            attrs(vec![
                attr(0, AttrValue::Str(Box::from(format!("h-{index}")))),
                attr(1, AttrValue::Int(2)),
            ]),
            TextBlock::new(vec![run("Título ", MarkSet::empty()), run("rico", MarkSet::single(Mark::new(schema.mark_id("strong").expect("strong"))))]),
        )));
        blocks.push(paragraph(schema, make_text(schema), None));
        blocks.push(Arc::new(Block::container(
            list,
            Attrs::empty(),
            vec![
                Arc::new(Block::container(
                    item,
                    Attrs::empty(),
                    vec![paragraph(schema, TextBlock::new(vec![run("Item A", MarkSet::empty())]), None)],
                )),
                Arc::new(Block::container(
                    item,
                    Attrs::empty(),
                    vec![paragraph(schema, TextBlock::new(vec![run("Item B", MarkSet::empty())]), None)],
                )),
            ],
        )));
        blocks.push(Arc::new(Block::container(
            quote,
            Attrs::empty(),
            vec![paragraph(schema, TextBlock::new(vec![run("Citação", MarkSet::empty())]), None)],
        )));
        blocks.push(Arc::new(Block::text(
            code,
            Attrs::empty(),
            TextBlock::new(vec![run("fn main() { println!(\"x\"); }", MarkSet::empty())]),
        )));
        blocks.push(Arc::new(Block::container(
            table,
            Attrs::empty(),
            vec![Arc::new(Block::container(
                row,
                Attrs::empty(),
                vec![
                    Arc::new(Block::container(
                        head,
                        Attrs::empty(),
                        vec![paragraph(schema, TextBlock::new(vec![run("Nome", MarkSet::empty())]), None)],
                    )),
                    Arc::new(Block::container(
                        cell,
                        Attrs::empty(),
                        vec![paragraph(schema, TextBlock::new(vec![run("Valor", MarkSet::empty())]), None)],
                    )),
                ],
            ))],
        )));
        blocks.push(Arc::new(Block::atom(separator, Attrs::empty(), AtomData::default())));
        blocks.push(paragraph(schema, TextBlock::new(vec![run("fim", MarkSet::empty())]), None));
    }
    Doc::new(doc_attrs, blocks, &rich_editor_auy::Limits::default()).expect("doc")
}

fn measure(label: &str, mut op: impl FnMut()) {
    for _ in 0..5 {
        op();
    }
    let start = Instant::now();
    for _ in 0..50 {
        op();
    }
    println!("{label}: {:.3} ms", start.elapsed().as_secs_f64() * 1000.0 / 50.0);
}

fn main() {
    let schema = rich_schema();
    let doc = make_document(&schema, 500);
    let json = doc.to_json_string(&schema);
    std::fs::write("/tmp/complex-all.json", json.as_bytes()).expect("write json");
    println!("blocks={} size={} json={}B", doc.blocks().len(), doc.size().0, json.len());
    let options = ParseOptions::default();
    measure("AUY all_types check_json", || {
        black_box(check_json(black_box(&json), &options).is_ok());
    });
    measure("AUY all_types from_json", || {
        black_box(from_json(black_box(&json), &schema, &options).expect("parse"));
    });
    measure("AUY all_types to_json", || {
        black_box(doc.to_json_string(&schema));
    });
    measure("AUY all_types validate", || {
        black_box(doc.validate(&schema));
    });
    measure("AUY all_types a11y_tree", || {
        black_box(doc.a11y_tree(&schema));
    });
    measure("AUY all_types a11y_validate", || {
        black_box(doc.validate_a11y(&schema));
    });
    let state = EditorState::from_doc(Arc::new(schema), doc);
    let mut transaction = state.transaction();
    transaction.insert_text(Pos(1), "x").expect("insert");
    measure("AUY all_types apply", || {
        black_box(state.apply(transaction.clone()).expect("apply"));
    });
}
