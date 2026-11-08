import { deepStrictEqual } from "node:assert/strict";
import { Document, fingerprint, init } from "./mod.ts";

Deno.test("wasm round-trip and fingerprint", async () => {
  await init();
  const doc = Document.fromJSON(
    '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Olá"}]}]}',
  );
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
