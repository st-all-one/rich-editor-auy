# Live demo: Rust → WASM → TypeScript ↔ PHP

A runnable page that exercises the real editor boundary, not a mock:

- **Rust** owns the canonical model, schema, steps, history, a11y tree, DOCX-safe
  `raw_code`, table algebra, `DocView`, and the CLI used by PHP.
- **TypeScript** runs the `@rich-editor-auy/core` WASM `Editor` and the
  `@rich-editor-auy/dom` adapter. The toolbar calls core commands; the DOM is only a projection.
- **PHP** serves the page, persists the canonical envelope, and validates/format it by invoking
  the Rust CLI through the `rich-editor-auy/php` transport classes.

## Run

```sh
./examples/demo/run.sh
# open http://127.0.0.1:8787/
```

`run.sh` builds `rich-editor-auy-cli`, bundles `main.ts` with Deno, copies the WASM binary, and
starts PHP. If `php` is absent locally, it starts the `php:8.4-cli` container with the repository
mounted (the Rust binary is reused from `target/debug/`).

## What the page demonstrates

- Full schema seed: headings, marks, lists, quote, code block, `raw_code`, table, figure, media,
  inline atoms, hard break, separator, refs, document metadata.
- Toolbar commands: undo/redo, marks, block types, lists, quote, lists, table, figure, separator,
  hard break, clipboard, history reset.
- IME composition and selection are handled by the DOM adapter and the stateful WASM editor.
- Live `DocView`, `A11yTree`, selection, change set, and canonical state panels.
- `Salvar PHP`: Rust validates the envelope, PHP writes `data/document.json`.
- `Validar Rust` and `Canonicalizar Rust`: PHP calls the Rust CLI and returns structured output.
- `Importar` / `Dry-run`: document, fragment, or operation batch import with `ProblemDetails`.
- `Reset seed`: drops the saved document and reloads the seed.

## API surface used by the page

| Route | Language | Purpose |
|---|---|---|
| `GET /api/document` | PHP | Saved envelope or seed document. |
| `POST /api/document` | PHP + Rust CLI | Validate and persist the envelope. |
| `POST /api/validate` | PHP + Rust CLI | Structured `ValidationReport`. |
| `POST /api/canonical` | PHP + Rust CLI | Canonical JSON bytes. |
| `POST /api/reset` | PHP | Remove saved state and return the seed. |
| `GET /api/health` | PHP | CLI path and executable check. |

The demo CSS belongs to the host page. The library itself still emits no `class` or `style`.
