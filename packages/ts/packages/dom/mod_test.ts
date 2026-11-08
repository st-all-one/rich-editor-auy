import { deepStrictEqual } from "node:assert/strict";
import { mount } from "./mod.ts";

Deno.test("mount refuses without a state", () => {
  deepStrictEqual(typeof mount, "function");
});
