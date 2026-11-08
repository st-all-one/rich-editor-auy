//! Exemplo mínimo real de `taino-edit-core` + `taino-edit-extensions`
//! rodando fora do browser (headless de verdade: nenhum `web-sys`).
//!
//! Cobre o caminho que o usuário percorre: schema → documento → edição via
//! `Transform` → negrito via `Keymap` → undo → persistência JSON/HTML/Markdown.
//!
//! Execução (a partir de `dossie/taino-edit/exemplos/`):
//!   cargo run -q

use taino_edit_core::{
    markdown::to_markdown, EditorState, Fragment, KeyPress, NodeSpec, SchemaBuilder, Selection,
    Slice,
};
use taino_edit_extensions::{
    build_keymap_with, build_schema_with, Bold, Heading, History, Italic, Paragraph,
};

fn main() {
    // 1) Schema: dois primitivos universais (doc/text) + extensões.
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
    let exts: Vec<&dyn taino_edit_extensions::Extension> =
        vec![&Paragraph, &Heading, &Bold, &Italic, &History];
    let schema = build_schema_with(base, &exts, "doc").expect("schema compila");

    // 2) Documento inicial montado à mão (h1 + parágrafo).
    let heading = schema
        .node(
            "heading",
            [("level".to_string(), serde_json::json!(1u64))]
                .into_iter()
                .collect(),
            vec![schema.text("Título", vec![]).unwrap()],
            vec![],
        )
        .unwrap();
    let para = schema
        .node(
            "paragraph",
            Default::default(),
            vec![schema.text("Hello world", vec![]).unwrap()],
            vec![],
        )
        .unwrap();
    let doc = schema
        .node("doc", Default::default(), vec![heading, para], vec![])
        .unwrap();
    let state = EditorState::new(doc, schema.clone());
    println!("inicial     → {}", state.doc().to_html());

    // 3) Edição tipada: inserir "!" no fim do parágrafo.
    //    Posições: h1 ocupa 1 + 6 + 1 = 8; parágrafo abre em 8, texto vai de
    //    9 a 20; logo a posição antes do token de fechamento é 20.
    let mut tx = state.tr();
    let bang = schema.text("!", vec![]).unwrap();
    tx.transform()
        .insert(20, Slice::new(Fragment::from_node(bang), 0, 0), &schema)
        .expect("inserção válida");
    let state = state.apply(tx);
    println!("após editar → {}", state.doc().to_html());

    // 4) Negrito em "Hello" via keymap (Mod-b), como o adapter faz no browser.
    let mut tx = state.tr();
    tx.set_selection(Selection::Text {
        anchor: 9,
        head: 14,
    });
    let state = state.apply(tx);
    let keymap = build_keymap_with(&exts, &schema, /*mac=*/ false);
    let press = KeyPress {
        key: "b".into(),
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    };
    let mut next = None;
    {
        let mut dispatch = |t| next = Some(state.apply(t));
        keymap.handle(&state, &press, Some(&mut dispatch));
    }
    let state = next.expect("Mod-b aplicou");
    println!("após negrito→ {}", state.doc().to_html());

    // 5) Undo pelo mesmo keymap (Mod-z).
    let press_z = KeyPress {
        key: "z".into(),
        ctrl: true,
        ..press.clone()
    };
    let mut undone = None;
    {
        let mut dispatch = |t| undone = Some(state.apply(t));
        keymap.handle(&state, &press_z, Some(&mut dispatch));
    }
    let state = undone.expect("Mod-z aplicou");
    println!("após undo   → {}", state.doc().to_html());

    // 6) Persistência: JSON validado contra o schema, HTML e Markdown.
    let json = state.doc().to_json();
    let bytes = serde_json::to_string_pretty(&json).unwrap();
    let reparsed = schema.node_from_json(&json).expect("JSON volta ao schema");
    assert_eq!(&reparsed, state.doc());
    println!("json        → {bytes}");
    println!("html        → {}", state.doc().to_html());
    println!("markdown    → {:?}", to_markdown(state.doc()));

    // 7) O schema é o guarda-corpo: conteúdo inválido não é construído.
    let bad = schema.node(
        "paragraph",
        Default::default(),
        vec![schema.node("paragraph", Default::default(), vec![], vec![]).unwrap()],
        vec![],
    );
    println!("parágrafo com bloco dentro é rejeitado? {}", bad.is_err());
}
