import { deepStrictEqual } from "node:assert/strict";
import { basicSchema } from "./mod.ts";

Deno.test("basic preset is deterministic", () => {
  deepStrictEqual(basicSchema().name, "basic");
});
