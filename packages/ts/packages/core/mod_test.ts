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
  deepStrictEqual(editor.block(0)?.kind, "paragraph");
  deepStrictEqual(editor.block(99), null);
  deepStrictEqual(editor.view({ from: 0, to: 1 }).blocks.length, 1);
});

Deno.test("applied results omit the document and report blocks", async () => {
  await init();
  const editor = Editor.fromJSON(PARAGRAPH);
  const applied = editor.apply({
    ops: [{ op: "insert_text", at: 1, text: "X" }],
  });
  deepStrictEqual("doc" in applied, false);
  deepStrictEqual(applied.changes.blocks.from, 0);
  deepStrictEqual(applied.changes.blocks.to, 1);
  deepStrictEqual(applied.changes.insertedBlocks, []);
  deepStrictEqual(applied.changes.removedBlocks, []);
  const split = editor.apply({ ops: [{ op: "split_block", at: 3 }] });
  deepStrictEqual(split.changes.insertedBlocks, [1]);
  const changed = editor.setBlockType("heading", { level: 2 });
  deepStrictEqual("doc" in changed, false);
  deepStrictEqual(editor.view().blocks[0]?.kind, "heading");
});

Deno.test("selection commands and position conversion round-trip", async () => {
  await init();
  const editor = Editor.fromJSON(PARAGRAPH);
  editor.setSelection(5, 5);
  const applied = editor.insertText("!");
  deepStrictEqual("doc" in applied, false);
  deepStrictEqual(editor.doc().plainText(), "Olá!");
  deepStrictEqual(editor.selection().type, "caret");
  deepStrictEqual(editor.block(0)?.index, 0);
  deepStrictEqual(editor.posToUtf16(3), 2);
  deepStrictEqual(editor.utf16ToPos(0, 2), 3);
  deepStrictEqual(editor.snapToScalar(3), 3);
  deepStrictEqual(editor.blockAt(2)?.index, 0);
  deepStrictEqual(editor.blockRange(0)?.from, 0);
  deepStrictEqual(editor.validate().ok, true);
  deepStrictEqual(editor.validateA11yReport().ok, true);
  deepStrictEqual(editor.canUndo(), true);
  const undone = editor.undo();
  deepStrictEqual(undone.applied, true);
  deepStrictEqual(editor.doc().plainText(), "Olá");
  deepStrictEqual(editor.canRedo(), true);
});

Deno.test("transactions compose, consume once, and reject stale bases", async () => {
  await init();
  const editor = Editor.fromJSON(PARAGRAPH);
  const transaction = editor.transaction();
  transaction.insertText(1, "A").insertText(2, "B");
  const applied = editor.apply(transaction);
  deepStrictEqual("doc" in applied, false);
  deepStrictEqual(transaction.consumed(), true);
  deepStrictEqual(editor.doc().plainText(), "ABOlá");
  let reused = false;
  try {
    editor.applyTransaction(transaction);
  } catch (error) {
    reused = error instanceof DocumentError && error.code ===
        "consumed_transaction";
  }
  deepStrictEqual(reused, true);
  const stale = editor.transaction();
  editor.apply({ ops: [{ op: "insert_text", at: 1, text: "Z" }] });
  let rejected = false;
  try {
    editor.applyTransaction(stale);
  } catch (error) {
    rejected = error instanceof DocumentError && error.code === "stale_batch";
  }
  deepStrictEqual(rejected, true);
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

Deno.test("import reports correlation, accepted blocks, and anchor policies", async () => {
  await init();
  const editor = new Editor();
  const report = editor.import({
    mode: "document",
    requestId: "req-7",
    source: "panel-a",
    destination: "note-9",
    schema: editor.schemaFingerprint(),
    doc: JSON.parse(PARAGRAPH) as Record<string, unknown>,
  });
  deepStrictEqual(report.requestId, "req-7");
  deepStrictEqual(report.source, "panel-a");
  deepStrictEqual(report.destination, "note-9");
  deepStrictEqual(report.schema, editor.schemaFingerprint());
  deepStrictEqual(report.acceptedBlocks, 1);
  deepStrictEqual(report.blocks, 1);

  const anchored = {
    type: "doc",
    content: [{
      type: "paragraph",
      attrs: { id: "anchor-1" },
      content: [{ type: "text", text: "kept" }],
    }],
  };
  const rejected = editor.import({
    mode: "document",
    doc: anchored,
    options: { idPolicy: "reject" },
  });
  deepStrictEqual(rejected.accepted, false);
  deepStrictEqual(rejected.problems[0]?.code, "id_rejected");

  const regenerated = editor.import({
    mode: "document",
    doc: anchored,
    options: { idPolicy: "regenerate", idSeed: "seed-1" },
  });
  deepStrictEqual(regenerated.accepted, true);
  deepStrictEqual(regenerated.idMap.length, 1);
  deepStrictEqual(regenerated.idMap[0]?.from, "anchor-1");
  const first = regenerated.idMap[0]?.to;
  deepStrictEqual(typeof first, "string");
  deepStrictEqual(first?.length, 64);

  const again = editor.import({
    mode: "document",
    doc: anchored,
    options: { idPolicy: "regenerate", idSeed: "seed-1" },
  });
  deepStrictEqual(again.idMap[0]?.to, first);

  const missingSeed = editor.import({
    mode: "document",
    doc: anchored,
    options: { idPolicy: "regenerate" },
  });
  deepStrictEqual(missingSeed.accepted, false);
  deepStrictEqual(missingSeed.problems[0]?.code, "invalid_id_policy");

  const unknownPolicy = editor.import({
    mode: "document",
    doc: anchored,
    options: { idPolicy: "shuffle" as "preserve" },
  });
  deepStrictEqual(unknownPolicy.accepted, false);
  deepStrictEqual(unknownPolicy.problems[0]?.code, "invalid_id_policy");

  const limited = editor.import({
    mode: "document",
    doc: {
      type: "doc",
      content: [
        { type: "paragraph", content: [{ type: "text", text: "a" }] },
        { type: "paragraph", content: [{ type: "text", text: "b" }] },
      ],
    },
    options: { limits: { maxBlocks: 1 } },
  });
  deepStrictEqual(limited.accepted, false);
  deepStrictEqual(limited.problems[0]?.code, "limit_exceeded");
  deepStrictEqual(limited.problems[0]?.instance, "blocks");

  const bytes = editor.import({
    mode: "document",
    doc: JSON.parse(PARAGRAPH) as Record<string, unknown>,
    options: { limits: { maxBytes: 4 } },
  });
  deepStrictEqual(bytes.accepted, false);
  deepStrictEqual(bytes.problems[0]?.instance, "payload");
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

Deno.test("clipboard copy and paste round-trip through the slice wire", async () => {
  await init();
  const editor = Editor.fromJSON(
    '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"abcd"}]}]}',
  );
  const slice = editor.slice(2, 4);
  deepStrictEqual(slice.text, "bc");
  const pasted = editor.insertSlice(1, slice);
  deepStrictEqual(pasted.changes?.selection, true);
  deepStrictEqual(editor.doc().plainText(), "bcabcd");
  const cut = editor.cutRange(1, 3);
  deepStrictEqual(cut.slice.text, "bc");
  deepStrictEqual(editor.doc().plainText(), "abcd");
  const undone = editor.undo();
  deepStrictEqual(undone.applied, true);
  deepStrictEqual(editor.doc().plainText(), "bcabcd");
});

Deno.test("composition is one undo group through the TS wrapper", async () => {
  await init();
  const editor = Editor.fromJSON(
    '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"hi"}]}]}',
  );
  editor.beginComposition(1, 1);
  deepStrictEqual(editor.isComposing(), true);
  editor.updateComposition("あ");
  editor.commitComposition();
  deepStrictEqual(editor.isComposing(), false);
  deepStrictEqual(editor.doc().plainText(), "あhi");
  editor.undo();
  deepStrictEqual(editor.doc().plainText(), "hi");
});

Deno.test("dryRun validates documents and fragments without mutating", async () => {
  await init();
  const editor = new Editor();
  const before = editor.blocks();
  const accepted = editor.dryRun({
    mode: "document",
    doc: JSON.parse(PARAGRAPH) as Record<string, unknown>,
  });
  deepStrictEqual(accepted.accepted, true);
  deepStrictEqual(accepted.problems.length, 0);
  deepStrictEqual(editor.blocks(), before);
  deepStrictEqual(editor.version(), 0);

  const fragment = editor.dryRun({
    mode: "insert",
    at: 0,
    fragment: {
      blocks: [{
        type: "paragraph",
        content: [{ type: "text", text: "X" }],
      }],
    },
  });
  deepStrictEqual(fragment.accepted, true);
  deepStrictEqual(editor.blocks(), before);

  const rejected = editor.dryRun({
    mode: "document",
    doc: { type: "doc", content: [{ type: "chart" }] },
  });
  deepStrictEqual(rejected.accepted, false);
  deepStrictEqual(rejected.problems[0]?.code, "unknown_token");
});

Deno.test("import honors addToHistory and batch limits", async () => {
  await init();
  const silent = new Editor();
  const report = silent.import({
    mode: "document",
    doc: JSON.parse(PARAGRAPH) as Record<string, unknown>,
    options: { addToHistory: false },
  });
  deepStrictEqual(report.accepted, true);
  deepStrictEqual(silent.canUndo(), false);

  const loud = new Editor();
  const applied = loud.import({
    mode: "document",
    doc: JSON.parse(PARAGRAPH) as Record<string, unknown>,
  });
  deepStrictEqual(applied.accepted, true);
  deepStrictEqual(loud.canUndo(), true);

  const limited = new Editor();
  const ops = limited.import({
    mode: "ops",
    ops: [
      { op: "insert_text", at: 1, text: "a" },
      { op: "insert_text", at: 2, text: "b" },
    ],
    options: { limits: { maxOps: 1 } },
  });
  deepStrictEqual(ops.accepted, false);
  deepStrictEqual(ops.problems[0]?.code, "limit_exceeded");
  deepStrictEqual(ops.problems[0]?.instance, "ops");

  const table = new Editor();
  const cells = table.import({
    mode: "document",
    doc: {
      type: "doc",
      content: [{
        type: "table",
        content: [{
          type: "table_row",
          content: [{
            type: "table_cell",
            content: [{ type: "paragraph", content: [] }],
          }],
        }],
      }],
    },
    options: { limits: { maxTableCells: 0 } },
  });
  deepStrictEqual(cells.accepted, false);
  deepStrictEqual(cells.problems[0]?.code, "limit_exceeded");
  deepStrictEqual(cells.problems[0]?.instance, "tables");
});
