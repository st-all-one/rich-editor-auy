<div align="center">

# rich-editor-auy

**A headless, deterministic, accessible-by-construction rich-text engine for Rust.**
Structured content, invertible edits, canonical bytes, semantic accessibility — no DOM, no runtime,
no UI opinions.

`Rust 1.99` pin · `MSRV 1.92` · edition 2024 · `no_std + alloc` · `MIT OR Apache-2.0`

</div>

> **Status:** MVP implementation (F0–F3.5 materialized; wire/limits/a11y hardening applied; the
> S2 wire and accessibility targets are met and the hot path carries a calibrated allocation
> ceiling). The Rust core,
> WASM artifact, Deno/JSR packages, DOM adapter, Composer package and CLI are present. `make check`
> is the local gate; measured performance deviations and open design items live in
> [`DIVERGENCES.md`](DIVERGENCES.md). The planning documents are written in Portuguese; the public
> API and rustdoc are in English.

**AUY** stands for **accessibility + universality**: universal accessibility is a core invariant,
not an adapter option. The core projects a deterministic semantic tree; the adapter maps it to
native/ARIA and the host audits the rendered experience.

---

## What it is

`rich-editor-auy` treats a rich-text document as a **canonical immutable value with an algebra of
editing**:

- a tree of blocks with **flat inline content** (marks are run metadata, not nodes);
- a schema that validates content expressions, marks, attributes and invariants;
- small **steps** that can be applied, inverted and mapped (the basis for undo, history,
  decorations and collaboration);
- atomic transactions that produce a new state and a compact change set for the UI;
- deterministic, byte-stable JSON (plus a versioned envelope with a schema fingerprint);
- a deterministic **semantic accessibility tree** (`A11yTree`): roles, names, `alt`, `lang`/`dir`,
  reading order and validation live in the core, not in the UI;
- zero `unsafe`, zero panics on untrusted input, one audited normal dependency (`blake3`,
  `no_std`/`pure`) and no runtime or UI dependency.

A UI — web, canvas, native, terminal, server — is a client: it translates events, lays out, paints
and owns I/O. The core owns the document, the edit algebra, the bytes and the **semantic
accessibility tree**; the adapter maps that tree to native elements/ARIA; the host audits contrast,
focus and WCAG. Accessibility is a core invariant, not an optional adapter feature. The library
never styles that UI: no CSS, no classes, no wrapper.

## What it is not

Not a component, not a framework, not a DOM `contenteditable` wrapper, not a layout engine, not a
CSS theme, not a sanitizer, not a persistence layer, not a collaboration server. It is also not an
accessibility overlay: semantic roles, names and `alt` live in the document and schema, not patched
into the DOM after the fact. Everything else (layout, styling, persistence, collaboration) is a
consumer concern or a separate crate (post-MVP).

## Ingestion contract

The wire vocabulary is the complete schema (`Schema::full()`). Every incoming token **must** map to
a declared node, mark, atom or attribute; raw XML, HTML or arbitrary JSON is never accepted as a
document subtree. A converter that cannot map a token has exactly three options:

1. map it to an existing semantic node;
2. represent it as declared data (for example an image asset);
3. reject it with a structured RFC 9457 `ProblemDetails` error.

Unknown tokens, marks and attributes produce `ProblemDetails` with `type`, `title`, `status`,
`detail`, `instance`, `code` and `suggestion`. Documented conversions:

| Source feature | Decision |
|---|---|
| Chart, graph | Render to an image (`image` / `image_block`) with `alt` and caption |
| Formula, equation | Render to an image; formula source goes in `alt` or `raw_code` |
| Page break | Not representable: error `unrepresentable_feature` |
| Comment, tracked change | Not representable in the default schema: reject or accept/reject first |
| Text box | Flatten into paragraphs/blockquote in reading order |
| XML, HTML, code, any verbatim payload | `raw_code` with `text*`, `language`, `format`, `filename`, `meta`, `line_numbers`, and `CodeBlock` a11y |

`raw_code` is a block, not an escape hatch around validation: its text is still a normal text
model, its attributes are schema-declared, and it projects as a `CodeBlock` with a language.

## Design at a glance

```mermaid
graph LR
  D["Doc<br/>blocks + inline runs"] --> S["EditorState<br/>doc + selection + marks + history"]
  S --> T["Transaction<br/>steps + metadata"]
  T --> D
  S --> V["view data<br/>blocks, runs, ranges, change set"]
  V --> U["consumer UI"]
  U -->|intent| C["command"]
  C --> T
```

- **Document:** block tree; each textblock holds a flat sequence of runs with a mark set.
- **Positions:** UTF-8 byte offsets plus one token per structural open/close/atom; the UI converts
  to its own units (UTF-16, graphemes) explicitly.
- **Editing:** closed `Step` enum (`Replace`, `ReplaceAround`, `AddMark`, `RemoveMark`,
  `SetNodeAttr`, `SetDocAttr`, `ReplaceDoc`, `SetInlineAttr`, `SetMarkAttr`) with
  `apply`/`invert`/`map`.
- **Ownership:** `Arc` + copy-on-write; the common path mutates in place when uniquely owned.
- **Bytes:** compact canonical JSON, schema-ordered attributes and marks, no floats, no `HashMap`.
- **Bytes and hashes:** BLAKE3-256 is the only project hash (schema fingerprint, envelope digest,
  integrity), with domain separation; no FNV or SHA-256.
- **Rigor:** `forbid(unsafe_code)`, deny `unwrap`/`expect`/`panic` even in tests, no clock/env/RNG
  in the core, 300-line file gate, allocation/instruction budgets, package/publish gates.

## Integration

Canonical JSON is the cross-language ABI. PHP stores and transports bytes; TypeScript runs the core
as WASM. Neither SDK reimplements the schema or the edit algebra. The official distribution is a
crates.io crate, a WASM artifact, a Composer package and a Deno 2.0/JSR workspace. Release order is
`rich-editor-auy` → `rich-editor-auy-macros` → `rich-editor-auy-wasm` → JSR/Composer, because the
WASM crate depends on the published core.

- **TypeScript/Deno 2.0:** `@rich-editor-auy/core` (WASM, no DOM), `@rich-editor-auy/dom`
  (required adapter for UI; it does not reimplement semantics), `@rich-editor-auy/schema`
  (`fullSchema` and `SCHEMA_FINGERPRINT`). The packages are published to **JSR** from `packages/ts`.
- **PHP:** the thin Composer package `rich-editor-auy/php` for envelope, byte-preserving documents
  and optional CLI validation, returning a `ValidationReport` with issue codes, offsets and paths.
  No HTML renderer, no schema engine.
- **Zero styling:** the adapter uses semantic native elements by default (`neutral` is opt-in),
  `data-auy-*` and an `A11yTree` mapping. No CSS, no `class`, no `style`, no wrapper; `destroy()`
  restores the host.
- **Render data:** `Editor.view()` returns a `DocView` with schema names, byte ranges, inline runs,
  atoms, attributes and nested children; it is separate from the semantic `A11yTree`.
- **References:** `DocId`/`NodeId` + typed `RefTarget`; the core preserves and validates shape, the
  optional `refs` crate resolves a corpus, backlinks and dangling targets.
- **Accessibility (AUY):** the core supplies the semantic tree, names/`alt`/`lang`/`dir`/order and
  validation; the adapter maps roles to native/ARIA and renders the semantic tree; the host owns
  keyboard/focus/IME and dispatches core commands; the consumer owns contrast, focus styles and the
  final WCAG audit. There is no `aria: false` opt-out; `data`
  explicitly transfers rendering responsibility.

```ts
import { Editor, init } from "@rich-editor-auy/core";
import { mount } from "@rich-editor-auy/dom";

await init();
const editor = Editor.fromJSON(savedJSON);
const view = mount(host, {
  state: editor,
  label: "Documento",
  onInput: (_event, current) => save(current.doc().toJSON()),
});

editor.apply({ ops: [{ op: "insert_text", at: 0, text: "Olá" }] });
view.dispatch(editor);
```

`onInput` is an intent boundary: the adapter does not reimplement the edit algebra. The host
applies core commands and calls `editor.dispatch(nextState)`.

No CSS import. The host remains yours.

## Clipboard and IME

Copy, cut and paste are core operations, not adapter guesses. A slice carries inline runs with
their marks, or a detached block sequence with block anchors removed; a paste validates the slice
against the schema before applying it as one transaction.

```rust
use rich_editor_auy::Editor;
use rich_editor_auy::pos::{Pos, Range};

let mut editor = Editor::from_json(saved_json)?;
let copy = editor.slice(Range::new(Pos(1), Pos(5)))?;      // runs + marks, or whole blocks
editor.delete_range(Range::new(Pos(1), Pos(5)))?;          // caret lands where content was
editor.insert_slice(Pos(1), &copy)?;                       // one history entry
let wire = copy.to_json_string(editor.schema());           // canonical slice JSON
```

| Operation | Inline range | Range across blocks |
|---|---|---|
| `slice` | runs and marks | partial edge blocks plus every complete block, no `id` |
| `insert_slice` | spliced into the host text block | host block splits first, so the caret keeps its place |
| `replace_slice` | one text block only | both ends must be block boundaries |
| `delete_range` | inline removal | surviving edge fragments merge, empty document gets the filler |

IME composition is one history group: `beginComposition` → `updateComposition` → `commitComposition`
(or `cancelComposition`), with `isComposing()` while it is open. The DOM adapter maps
`compositionstart`/`update`/`end` onto those calls, so undo never rewinds one Japanese character at
a time.

Renderer notes: `mount` defaults to native semantics (`<h2>`, `<ul>`, `<a href>`, `<img alt>`);
`semantics: "neutral"` renders `div`/`span` plus the role, name and level from the a11y tree. The
adapter maps the native selection to UTF-8 byte positions through `data-auy-from`/`data-auy-to`, and
announces remote changes in a polite live region.

## Declarative schemas

`rich-editor-auy-macros` carries an optional `schema!` front-end. It expands to builder calls, so
the runtime builder stays the single source of truth and every check runs in both paths.

```rust
use rich_editor_auy_macros::schema;

schema! {
    pub mod kinds;
    doc { content: "block+", attrs: { lang: str }, a11y: { role: document } }
    paragraph {
        group: "block",
        content: "text*",
        marks: "_",
        a11y: { role: paragraph, name: text }
    }
    heading {
        group: "block",
        content: "text*",
        marks: "_",
        attrs: { level: int(1..=6) = 1 },
        a11y: { role: heading(2), name: text, level: "level" }
    }
    text { inline: true, a11y: { role: text, name: text } }
    link { mark, attrs: { href: str required }, a11y: { role: link, name: text } }
}

let schema = kinds::build()?;
assert_eq!(schema.node_kind("heading"), Some(kinds::NODE_HEADING));
assert_eq!(kinds::NODE_HEADING_LEVEL.0, 1);
```

Without the leading `pub mod NAME;`, the macro is an expression (`let schema = schema! { ... };`).
Attributes accept `str`, `bool`, `ref`, `any`, `int(min..=max)`, `enum("a", "b")`, `list(type)`,
`map(type)` and an optional `required` marker; defaults are literals for `str`, `bool`, `int` and
`enum`. `a11y` declares the role, name policy, `alt` policy and the attribute that carries a
heading level. Invalid declarations fail at compile time, and `tests/ui` locks the messages.

## Planning dossier

| Document | Question |
|---|---|
| [`plan/00_panorama.md`](plan/00_panorama.md) | What is the thesis, scope and kill criteria? |
| [`plan/01_referencias.md`](plan/01_referencias.md) | What did ProseMirror, Lexical, Quill, TipTap, taino-edit, knudge and html-auy teach? |
| [`plan/02_decisoes.md`](plan/02_decisoes.md) | Which decisions are open, with options and recommendations? |
| [`plan/03_decisoes-fechadas.md`](plan/03_decisoes-fechadas.md) | What is already closed and cannot regress? |
| [`plan/04_arquitetura.md`](plan/04_arquitetura.md) | How do the pieces fit, with types and flows? |
| [`plan/05_desempenho.md`](plan/05_desempenho.md) | What is the performance budget and how is it measured? |
| [`plan/06_api-e-integracao.md`](plan/06_api-e-integracao.md) | What does a UI receive and what must it do? |
| [`plan/07_rigor.md`](plan/07_rigor.md) | Which toolchain, lints, CI and tests are mandatory? |
| [`plan/08_roadmap.md`](plan/08_roadmap.md) | In what order, with which acceptance and stop criteria? |
| [`DIVERGENCES.md`](DIVERGENCES.md) | Where does the implementation differ from the plan, and why? |
| [`plan/09_frontend-e-referencias.md`](plan/09_frontend-e-referencias.md) | How do PHP/TS integrate without CSS, and how do cross-publication references work? |
| [`plan/10_acessibilidade.md`](plan/10_acessibilidade.md) | What does AUY mean, which a11y contract is core, and how is it tested? |
| [`AGENTS.md`](AGENTS.md) | How does an agent/contributor operate in this repository? |

The raw exploration lives in [`plan/brainstorm/`](plan/brainstorm/): model alternatives and the
catalog of Unicode/selection/schema/wire edges with the test that must lock each one.

## License

Intended license: **MIT OR Apache-2.0** (decision F05). The seed repository currently carries the
Apache-2.0 text in [`LICENSE`](LICENSE); F0 adds `LICENSE-MIT` and `LICENSE-APACHE` before the first
public release.
