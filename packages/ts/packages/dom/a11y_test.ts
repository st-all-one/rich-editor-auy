import { deepStrictEqual, ok as or } from "node:assert/strict";
import { JSDOM } from "jsdom";
import { Editor, init } from "@rich-editor-auy/core";
import { mount } from "./mod.ts";

const DOCUMENT = `{"type":"doc","content":[
  {"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Título"}]},
  {"type":"paragraph","content":[
    {"type":"text","text":"Veja "},
    {"type":"text","text":"a referência","marks":[{"type":"link","attrs":{"href":"https://example.com"}}]}
  ]},
  {"type":"bullet_list","content":[
    {"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"um"}]}]},
    {"type":"list_item","content":[{"type":"paragraph","content":[{"type":"text","text":"dois"}]}]}
  ]}
]}`;

interface AxeViolation {
  readonly id: string;
  readonly impact: string;
  readonly nodes: readonly { readonly target: readonly string[] }[];
}

/** Global installed by evaluating the axe bundle inside the jsdom window. */
interface AxeRuntime {
  run(
    context: Element,
    options: unknown,
  ): Promise<{ readonly violations: readonly AxeViolation[] }>;
}

/** Loads the axe bundle once; the resolver needs the npm cache, not a bundler. */
let axeSource: string | undefined;

async function loadAxe(): Promise<string> {
  if (axeSource === undefined) {
    const path = import.meta.resolve("axe-core/axe.min.js");
    axeSource = await Deno.readTextFile(new URL(path));
  }
  return axeSource;
}

/** Gives jsdom the hit-testing hooks axe reads during setup. */
function polyfillHitTesting(window: Window): void {
  const targets = [window.document, window] as unknown as Record<
    string,
    unknown
  >[];
  for (const target of targets) {
    target.elementsFromPoint ??= () => [];
    target.elementFromPoint ??= () => null;
  }
}

/** Runs axe-core inside a jsdom window and returns its violations. */
async function audit(host: HTMLElement): Promise<readonly AxeViolation[]> {
  const window = host.ownerDocument.defaultView;
  if (window === null) return [];
  polyfillHitTesting(window);
  window.eval(await loadAxe());
  // `window.eval` installs `axe` on the jsdom global, outside TypeScript's view.
  const runtime = window as unknown as { axe?: AxeRuntime };
  if (runtime.axe === undefined) return [];
  const result = await runtime.axe.run(host, {
    // Contrast depends on the host stylesheet, which the adapter never writes.
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a"] },
    rules: { "color-contrast": { enabled: false } },
  });
  return result.violations;
}

Deno.test("mounted editor has no axe-core violations", async () => {
  await init();
  const dom = new JSDOM('<main><div id="host"></div></main>', {
    pretendToBeVisual: true,
    // `window.eval` must run inside the jsdom context so the bundle sees `document`.
    runScripts: "outside-only",
  });
  const host = dom.window.document.getElementById("host");
  if (host === null) return;
  const editor = Editor.fromJSON(DOCUMENT);
  const mounted = mount(host as unknown as HTMLElement, {
    state: editor,
    editable: false,
    label: "Documento",
  });
  const violations = await audit(host as unknown as HTMLElement);
  deepStrictEqual(
    Array.from(violations, (violation) => violation.id),
    [],
    `axe violations: ${JSON.stringify(violations)}`,
  );
  mounted.destroy();
});

Deno.test("neutral mode keeps the accessible name and level", async () => {
  await init();
  const dom = new JSDOM("<div id='host'></div>", {
    pretendToBeVisual: true,
    runScripts: "outside-only",
  });
  const host = dom.window.document.getElementById("host");
  if (host === null) return;
  const editor = Editor.fromJSON(DOCUMENT);
  const mounted = mount(host as unknown as HTMLElement, {
    state: editor,
    editable: false,
    semantics: "neutral",
  });
  const heading = host.querySelector("[role=heading]");
  deepStrictEqual(heading?.getAttribute("aria-level"), "2");
  const violations = await audit(host as unknown as HTMLElement);
  deepStrictEqual(
    Array.from(violations, (violation) => violation.id),
    [],
    `axe violations: ${JSON.stringify(violations)}`,
  );
  mounted.destroy();
});

Deno.test("axe reports a real violation so the audit is live", async () => {
  const dom = new JSDOM("<div id='host'><img src='x.png'></div>", {
    pretendToBeVisual: true,
    runScripts: "outside-only",
  });
  const host = dom.window.document.getElementById("host");
  if (host === null) return;
  const violations = await audit(host as unknown as HTMLElement);
  const ids = Array.from(violations, (violation) => violation.id);
  or(ids.includes("image-alt"), `expected image-alt in ${JSON.stringify(ids)}`);
});
