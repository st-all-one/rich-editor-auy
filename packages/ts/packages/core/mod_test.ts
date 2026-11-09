import { deepStrictEqual, throws } from "node:assert/strict";
import { Document, DocumentError, Editor, fingerprint, init } from "./mod.ts";

const PARAGRAPH =
  '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Olá"}]}]}';

Deno.test("wasm round-trip and fingerprint", async () => {
  await init();
  const doc = Document.fromJSON(PARAGRAPH);
  deepStrictEqual(doc.plainText(), "Olá");
  deepStrictEqual(doc.validateA11y(), []);
  const tree = doc.a11yTree();
  deepStrictEqual(tree.role, "document");
  deepStrictEqual(tree.children[0]?.role, "paragraph");
  deepStrictEqual(doc.insertText(1, "X").plainText(), "XOlá");
  deepStrictEqual(doc.splitBlock(2).plainText(), "Olá");
  deepStrictEqual(doc.deleteRange(1, 2).plainText(), "lá");
  deepStrictEqual(doc.toggleMark(1, 5, "strong").plainText(), "Olá");
  const envelope =
    `{"format":"rich-editor-auy","version":1,"schema_fingerprint":"${fingerprint()}","doc":${doc.toJSON()}}`;
  deepStrictEqual(Document.fromEnvelope({}, envelope).plainText(), "Olá");
  let rejected = false;
  try {
    Document.fromEnvelope({}, envelope.replace("rich-editor-auy", "other"));
  } catch {
    rejected = true;
  }
  deepStrictEqual(rejected, true);
});

Deno.test("stateful editor applies atomic batches", async () => {
  await init();
  const editor = Editor.fromJSON(PARAGRAPH);
  deepStrictEqual(editor.version(), 0);
  deepStrictEqual(editor.blocks(), 1);
  const applied = editor.apply({
    ops: [{ op: "insert_text", at: 1, text: "X" }],
  });
  deepStrictEqual(applied.version, 1);
  deepStrictEqual(editor.doc().plainText(), "XOlá");
  deepStrictEqual(applied.changes?.selection, true);
  const undone = editor.undo();
  deepStrictEqual(undone.applied, true);
  deepStrictEqual(editor.doc().plainText(), "Olá");
  const redone = editor.redo();
  deepStrictEqual(redone.applied, true);
  deepStrictEqual(editor.doc().plainText(), "XOlá");
  editor.clearHistory();
  deepStrictEqual(editor.history().can_undo, false);
  deepStrictEqual(editor.history().can_redo, false);
  const view = editor.view();
  deepStrictEqual(view.version, editor.version());
  deepStrictEqual(view.blocks[0]?.kind, "paragraph");
  deepStrictEqual(view.blocks[0]?.text, "XOlá");
  deepStrictEqual(view.blocks[0]?.runs?.[0]?.text, "XOlá");
  deepStrictEqual(view.blocks[0]?.range[0], 0);
});

Deno.test("rejected batches leave the editor unchanged", async () => {
  await init();
  const editor = Editor.fromJSON(PARAGRAPH);
  await throws(
    () =>
      editor.apply({
        ops: [
          { op: "insert_text", at: 1, text: "X" },
          { op: "delete_range", from: 99, to: 100 },
        ],
      }),
    (error: unknown) => {
      deepStrictEqual(error instanceof DocumentError, true);
      const problem = error as DocumentError;
      deepStrictEqual(problem.code, "out_of_bounds");
      deepStrictEqual(problem.status, 400);
      deepStrictEqual(problem.detail.includes("99"), true);
      return true;
    },
  );
  deepStrictEqual(editor.version(), 0);
  deepStrictEqual(editor.doc().plainText(), "Olá");
  const report = editor.dryRun({
    ops: [{ op: "insert_text", at: 1, text: "Y" }],
  });
  deepStrictEqual(report.accepted, true);
  deepStrictEqual(report.state.version, 0);
  deepStrictEqual(editor.doc().plainText(), "Olá");
});

Deno.test("import accepts documents, fragments, and rejects unknown tokens", async () => {
  await init();
  const editor = new Editor();
  const document = editor.import({
    mode: "document",
    schema: editor.schemaFingerprint(),
    doc: JSON.parse(PARAGRAPH) as Record<string, unknown>,
  });
  deepStrictEqual(document.accepted, true);
  deepStrictEqual(document.problems.length, 0);
  deepStrictEqual(document.state.doc.content, JSON.parse(PARAGRAPH).content);

  const fragment = editor.import({
    mode: "insert",
    at: 0,
    fragment: {
      blocks: [{
        type: "paragraph",
        content: [{ type: "text", text: "New" }],
      }],
    },
  });
  deepStrictEqual(fragment.accepted, true);
  deepStrictEqual(editor.blocks(), 2);

  const unknown = editor.import({
    mode: "document",
    doc: { type: "doc", content: [{ type: "chart" }] },
  });
  deepStrictEqual(unknown.accepted, false);
  deepStrictEqual(unknown.problems[0]?.code, "unknown_token");
  deepStrictEqual(unknown.problems[0]?.suggestion !== undefined, true);

  const stale = editor.import({
    mode: "ops",
    base_version: 0,
    ops: [{ op: "insert_text", at: 1, text: "Z" }],
  });
  deepStrictEqual(stale.accepted, false);
  deepStrictEqual(stale.problems[0]?.code, "stale_batch");
});

Deno.test("selection and attributes round-trip through the state", async () => {
  await init();
  const editor = Editor.fromJSON(PARAGRAPH);
  const selection = editor.setSelection(1, 3);
  deepStrictEqual(selection.selection, {
    type: "text",
    anchor: 1,
    head: 3,
    assoc: 1,
  });
  throws(
    () =>
      editor.apply({
        ops: [{ op: "set_block_attr", at: 1, name: "nope", value: 1 }],
      }),
    DocumentError,
  );
});

Deno.test("envelope and legacy state still use the full schema", async () => {
  await init();
  const editor = Editor.fromJSON(PARAGRAPH);
  deepStrictEqual(editor.schemaFingerprint(), fingerprint());
  const envelope =
    `{"format":"rich-editor-auy","version":1,"schema_fingerprint":"${fingerprint()}","doc":${
      JSON.stringify(editor.toJSON().doc)
    }}`;
  deepStrictEqual(Editor.fromEnvelope(envelope).doc().plainText(), "Olá");
});
