# Exemplos — taino-edit

Dois exemplos: um **executável e verificado** (Rust puro, sem browser) e um
componente Leptos mínimo (**não executado** neste ambiente — falta
`trunk`/wasm toolchain).

## 1. Rust headless — executado

`exemplos/src/main.rs` + `exemplos/Cargo.toml`, com dependências por caminho
para o checkout em `ref/taino-edit/` (nenhuma alteração em `ref/`).

```sh
cd dossie/taino-edit/exemplos
cargo run -q
```

Saída observada (reexecutada na revisão em 2026-11-07; idêntica à primeira
medição, byte a byte — colada abaixo por inteiro). Note que o JSON inclui o
heading com `"level": 1` e que **ambos** os blocos carregam
`"text_align": null`.

```
inicial     → <h1>Título</h1><p>Hello world</p>
após editar → <h1>Título</h1><p>Hello world!</p>
após negrito→ <h1>Título</h1><p><strong>Hello</strong> world!</p>
após undo   → <h1>Título</h1><p>Hello world!</p>
json        → {
  "content": [
    {
      "attrs": {
        "level": 1,
        "text_align": null
      },
      "content": [
        {
          "text": "Título",
          "type": "text"
        }
      ],
      "type": "heading"
    },
    {
      "attrs": {
        "text_align": null
      },
      "content": [
        {
          "text": "Hello world!",
          "type": "text"
        }
      ],
      "type": "paragraph"
    }
  ],
  "type": "doc"
}
html        → <h1>Título</h1><p>Hello world!</p>
markdown    → "# Título\n\nHello world!\n"
parágrafo com bloco dentro é rejeitado? true
```

O que o exemplo prova, ponto a ponto:

| Linha da saída | Prova |
| --- | --- |
| `inicial` | `SchemaBuilder` + `build_schema_with` montam um documento válido fora do browser |
| `após editar` | `Transaction::transform().insert` aplica um `ReplaceStep` e o doc novo é imutável |
| `após negrito` | `Keymap::handle` resolve `Ctrl-b` → `toggle_mark` → `AddMarkStep`, sem DOM |
| `após undo` | `History` guarda steps invertidos e `Mod-z` restaura o documento anterior |
| `json` | round-trip validado: `schema.node_from_json` devolve um doc igual (o `assert_eq!` no código passou) |
| `markdown` | `to_markdown` produz CommonMark |
| `rejeitado? true` | o autômato de conteúdo barra `paragraph` dentro de `paragraph` |

Observação de campo: o JSON carrega `"attrs": {"text_align": null}` mesmo em um
parágrafo que nunca foi alinhado — default da extensão `Align` vazando para o
documento serializado (`ref/taino-edit/crates/taino-edit-extensions/src/align.rs:20-27`).
Se formos copiar a ideia, atributos nulos não deveriam ser emitidos.

## 2. Componente Leptos mínimo — **não executado**

Este é o menor uso correto de `taino-edit-leptos`. Note que ele **passa um
keymap**: sem isso, Enter/Backspace/Delete são bloqueados e não fazem nada
(`ref/taino-edit/crates/taino-edit-leptos/src/lib.rs:336-339`).

```rust
use leptos::prelude::*;
use taino_edit_extensions::{build_keymap_with, build_schema_with, Bold, Italic, Paragraph};
use taino_edit_leptos::{EditorState, NodeSpec, SchemaBuilder, TainoEditor};

#[component]
fn App() -> impl IntoView {
    let base = SchemaBuilder::new()
        .node("doc",  NodeSpec { content: Some("block+".into()),  ..Default::default() })
        .node("text", NodeSpec { group:   Some("inline".into()),  ..Default::default() });

    let exts: Vec<&dyn taino_edit_extensions::Extension> = vec![&Paragraph, &Bold, &Italic];
    let schema = build_schema_with(base, &exts, "doc").expect("schema builds");

    let txt  = schema.text("Hello from Rust!", vec![]).unwrap();
    let para = schema.node("paragraph", Default::default(), vec![txt], vec![]).unwrap();
    let doc  = schema.node("doc", Default::default(), vec![para], vec![]).unwrap();

    let state  = RwSignal::new(EditorState::new(doc, schema.clone()));
    let keymap = build_keymap_with(&exts, &schema, /*mac=*/ false);

    view! { <TainoEditor state=state keymap=keymap /> }
}
```

Para rodar seria preciso: um crate `leptos = "0.8.3"` com feature `csr`,
`wasm32-unknown-unknown`, `trunk` e um `index.html` com
`<link data-trunk rel="rust" />`. Não fiz isso aqui: instalar `trunk` e a árvore
de dependências do Leptos está fora do escopo barato deste dossiê. O demo
completo e buildável do projeto é `ref/taino-edit/examples/basic-leptos/`
(412 linhas, toolbar com 13 extensões, tabelas, busca com decoration inline e
painéis de JSON/HTML ao vivo).

## 3. Versão "crua" sem adapter (core + dom)

Se o alvo não for Leptos, o caminho é `EditorView` direto:

```rust
use taino_edit_dom::EditorView;
let view = EditorView::mount(doc, schema, element);   // element: web_sys::Element
// a cada evento 'input':
if let Some(tr) = view.read_dom_changes() { /* aplicar ao seu estado */ }
// a cada mudança de estado:
view.update(new_doc);
view.set_selection(new_selection);
```

As três chamadas acima são exatamente o que o adapter faz
(`ref/taino-edit/crates/taino-edit-leptos/src/lib.rs:120-190,300-340`). Não
executado (exige wasm + DOM real).

## 4. Reproduzir a verificação

```sh
# testes host (executados por mim: 212 + 6)
cd ref/taino-edit
CARGO_TARGET_DIR=/tmp/taino-target cargo test -p taino-edit-core -p taino-edit-extensions
CARGO_TARGET_DIR=/tmp/taino-target cargo test -p taino-edit-dom --lib

# exemplo headless do próprio projeto (executado)
CARGO_TARGET_DIR=/tmp/taino-target cargo run -p headless-core

# testes de browser (NÃO executados: precisam de chromedriver + Chrome-for-Testing
# 148 e do runner wasm-bindgen-cli patchado)
./scripts/install-wasm-test-runner.sh
./scripts/wasm-test.sh
```
