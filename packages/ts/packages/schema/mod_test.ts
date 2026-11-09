import { deepStrictEqual } from "node:assert/strict";
import { fullSchema } from "./mod.ts";

Deno.test("full schema declares every core node and mark", () => {
  const schema = fullSchema();
  deepStrictEqual(schema.name, "full");
  for (const node of ["paragraph", "raw_code", "table", "figure", "image"]) {
    deepStrictEqual(schema.nodes.includes(node), true);
  }
  deepStrictEqual(schema.marks.includes("link"), true);
  deepStrictEqual(schema.nodes.includes("table_caption"), true);
});
