import { Document, init } from "../../packages/ts/packages/core/mod.ts";

await init();
const doc = Document.fromJSON(
  '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Olá"}]}]}',
);
console.log(doc.plainText());
console.log(doc.a11yTree().role);
