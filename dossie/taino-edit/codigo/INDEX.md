# Código copiado — origem, motivo e licença

Todos os arquivos abaixo são **cópias integrais, sem edição**, de
`ref/taino-edit/` (v0.7.0, commit `90ba5ed82097b2c7b5fc8158531e090ef89f3b4a`).
A revisão reconfirmou com `diff -q` que os seis `.rs` são byte a byte iguais às
origens e que as contagens de linha batem (312, 338, 605, 321, 112, 1016).
Nenhum arquivo-fonte do projeto traz cabeçalho de licença individual; a licença
é declarada na raiz do repositório (`LICENSE-MIT`, `LICENSE-APACHE`) e no
manifesto do workspace:

```toml
# ref/taino-edit/Cargo.toml:17
license = "MIT OR Apache-2.0"
```

Licença dupla **MIT OR Apache-2.0** permite cópia e reuso com atribuição. Se
algum trecho for reaproveitado no `rich-editor-auy`, preserve este aviso e o
arquivo `LICENSE-*` correspondente.

## Índice

| Arquivo local | Origem | Linhas | Por que foi copiado |
| --- | --- | --- | --- |
| `node.rs` | `crates/taino-edit-core/src/node.rs` | 312 | O modelo de documento: `Node = Arc<NodeInner>`, `NodeType`, `node_size` (posições em scalars), `same_markup`, `cut`, `PartialEq` com atalho por `Arc::ptr_eq`. É a peça que define ownership e copy-on-write. |
| `schema.rs` | `crates/taino-edit-core/src/schema.rs` | 338 | `SchemaBuilder` → `Schema` imutável: atribuição de ids, resolução de grupos, compilação das expressões de conteúdo, construção validada (`node`) vs. não validada (`create_node`), preenchimento de defaults. Mostra como o schema é a única fonte de verdade estrutural. |
| `step.rs` | `crates/taino-edit-core/src/step.rs` | 605 | O contrato de mudança atômica (`apply`/`invert`/`map`/`to_json`) e os cinco steps concretos. É o ponto de extensão para undo, rebase e futura colaboração — a decisão arquitetural mais consequente do projeto. |
| `state.rs` | `crates/taino-edit-core/src/state.rs` | 321 | `EditorState`, `Transaction` e `History`: como seleção, plugins e histórico se combinam; onde o histórico é clonado e invertido a cada transação (o custo escondido); o curto-circuito `HistoryIntent`. |
| `selection.rs` | `crates/taino-edit-core/src/selection.rs` | 112 | O enum `Selection` (Text/Node/Cell/All) e o mapeamento posicional com degradação para caret. Pequeno e completo: serve de especificação para a nossa seleção. |
| `view.rs` | `crates/taino-edit-dom/src/view.rs` | 1016 | O renderer inteiro: `mount`, `render` recursivo, `patch_children`/`try_patch`, `read_selection`/`set_selection`, `read_dom_changes` (o elo frágil), colagem, IME, `pos_at_point`, decorações e o `<br>` sintético. **Não é `core`** — está aqui porque a pergunta "como esse editor realmente edita" só se responde neste arquivo. |

## O que ficou de fora (e onde olhar)

| Tema | Arquivo de origem |
| --- | --- |
| Algoritmo de replace (2-way/3-way) | `crates/taino-edit-core/src/replace.rs` (316 linhas) |
| Autômato de conteúdo (NFA→DFA) | `crates/taino-edit-core/src/content.rs` (491 linhas) |
| `StepMap`/`Mapping` com mirror/recover | `crates/taino-edit-core/src/map.rs` (267 linhas) |
| `Fragment` com size cacheado | `crates/taino-edit-core/src/fragment.rs` (199 linhas) |
| `ResolvedPos` | `crates/taino-edit-core/src/pos.rs` (190 linhas) |
| Comandos (e a ausência de stored marks) | `crates/taino-edit-core/src/commands.rs` (600 linhas) |
| Wiring de eventos do adapter Leptos | `crates/taino-edit-leptos/src/lib.rs` (496 linhas) |
| Serializador SSR (contrato de markup) | `crates/taino-edit-dom/src/view_html.rs` (352 linhas) |
| Mapa DOM ↔ documento | `crates/taino-edit-dom/src/position_map.rs` (162 linhas) |
| Tabelas span-aware | `crates/taino-edit-extensions/src/table.rs` (1469 linhas) |

## Como ler

1. `node.rs` → `schema.rs`: entenda o que é um documento válido.
2. `step.rs` → `state.rs`: entenda como ele muda e como se desfaz.
3. `selection.rs`: entenda o que "onde estou" significa.
4. `view.rs`: entenda como isso vira DOM e como o DOM volta a ser modelo —
   prestando atenção em `read_dom_changes` e no `prevent_default` estrutural,
   que estão no adapter, não aqui.
