import { deepStrictEqual, ok, strictEqual } from "node:assert/strict";
import { JSDOM } from "jsdom";
import { Editor, init } from "@rich-editor-auy/core";
import { createAnnouncer, describeChange } from "./announce.ts";
import { mount } from "./mod.ts";
import {
  documentSize,
  offsetForBytes,
  pointAt,
  positionAt,
  textSpans,
} from "./positions.ts";
import { readSelection, writeSelection } from "./selection.ts";

const PARAGRAPH =
  '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Olá"}]}]}';

/** Builds a jsdom document and one empty host element. */
function fixture(): { dom: JSDOM; host: HTMLElement } {
  const dom = new JSDOM('<div id="host"></div>', { pretendToBeVisual: true });
  const host = dom.window.document.getElementById("host");
  ok(host !== null, "host must exist");
  return { dom, host: host as unknown as HTMLElement };
}

Deno.test("byte offsets map to UTF-16 offsets", () => {
  strictEqual(offsetForBytes("aé", 1), 1);
  strictEqual(offsetForBytes("aé", 3), 2);
  strictEqual(offsetForBytes("🙂", 4), 2);
  strictEqual(offsetForBytes("🙂", 2), 2);
});

Deno.test("rendered text spans keep absolute byte ranges", async () => {
  await init();
  const { host } = fixture();
  const editor = Editor.fromJSON(PARAGRAPH);
  const mounted = mount(host, { state: editor, editable: false });
  const spans = textSpans(host);
  strictEqual(spans.length, 1);
  strictEqual(spans[0]?.start, 1);
  strictEqual(spans[0]?.end, 5, "é is two UTF-8 bytes");
  strictEqual(documentSize(host), 6);
  const point = pointAt(host, 3);
  ok(point !== undefined, "position must map to a point");
  strictEqual(positionAt(point), 3);
  mounted.destroy();
});

Deno.test("selection round-trips through document positions", async () => {
  await init();
  const { host } = fixture();
  const editor = Editor.fromJSON(PARAGRAPH);
  const mounted = mount(host, { state: editor, editable: false });
  ok(writeSelection(host, { from: 1, to: 3 }), "selection must be written");
  const read = readSelection(host);
  ok(read !== undefined, "selection must be readable");
  deepStrictEqual(read, { from: 1, to: 3 });
  mounted.destroy();
});

Deno.test("insert intent goes through the core editor", async () => {
  await init();
  const { host } = fixture();
  const editor = Editor.fromJSON(PARAGRAPH);
  const mounted = mount(host, { state: editor, editor });
  ok(writeSelection(host, { from: 1, to: 1 }), "caret must be placed");
  host.dispatchEvent(
    new host.ownerDocument.defaultView!.InputEvent("beforeinput", {
      inputType: "insertText",
      data: "X",
      bubbles: true,
      cancelable: true,
    }),
  );
  strictEqual(editor.doc().plainText(), "XOlá");
  strictEqual(host.textContent, "XOlá");
  mounted.destroy();
});

Deno.test("destroy restores the host and removes the live region", async () => {
  await init();
  const { host } = fixture();
  host.innerHTML = "<span>before</span>";
  const editor = Editor.fromJSON(PARAGRAPH);
  const mounted = mount(host, { state: editor, live: true });
  strictEqual(host.getAttribute("role"), "textbox");
  const live = host.parentElement?.querySelector("[aria-live=polite]");
  ok(live !== null && live !== undefined, "live region must exist");
  mounted.destroy();
  strictEqual(host.innerHTML, "<span>before</span>");
  strictEqual(host.getAttribute("role"), null);
  strictEqual(host.parentElement?.querySelector("[aria-live=polite]"), null);
});

Deno.test("rendered tree carries data attributes and no styling", async () => {
  await init();
  const { host } = fixture();
  const editor = Editor.fromJSON(PARAGRAPH);
  const mounted = mount(host, { state: editor, editable: false });
  strictEqual(host.querySelector("style"), null);
  strictEqual(host.querySelector("[style]"), null);
  strictEqual(host.querySelector("[class]"), null);
  const paragraph = host.querySelector("p");
  ok(paragraph !== null, "native semantics must render a paragraph");
  strictEqual(paragraph.dataset.auyRole, "paragraph");
  strictEqual(paragraph.dataset.auyFrom, "0");
  strictEqual(paragraph.dataset.auyTo, "6");
  mounted.destroy();
});

Deno.test("neutral mode renders roles without native elements", async () => {
  await init();
  const { host } = fixture();
  const editor = Editor.fromJSON(
    '{"type":"doc","content":[{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"H"}]}]}',
  );
  const mounted = mount(host, {
    state: editor,
    editable: false,
    semantics: "neutral",
  });
  strictEqual(host.querySelector("h2"), null);
  const heading = host.querySelector("[role=heading]");
  ok(heading !== null, "neutral heading must exist");
  strictEqual(heading.getAttribute("aria-level"), "2");
  mounted.destroy();
});

Deno.test("announcer describes remote changes", () => {
  const { dom } = fixture();
  const host = dom.window.document.createElement("div");
  dom.window.document.body.append(host);
  const announcer = createAnnouncer(host as unknown as HTMLElement);
  announcer.announce("Document replaced");
  const region = dom.window.document.querySelector("[aria-live=polite]");
  strictEqual(region?.textContent, "Document replaced");
  strictEqual(describeChange(undefined), undefined);
  strictEqual(
    describeChange({
      blocks: { from: 0, to: 0 },
      ranges: [],
      selection: false,
      full: true,
      insertedBlocks: [],
      removedBlocks: [],
    }),
    "Document replaced",
  );
  strictEqual(
    describeChange({
      blocks: { from: 0, to: 1 },
      ranges: [],
      selection: true,
      full: false,
      insertedBlocks: [],
      removedBlocks: [],
    }),
    undefined,
  );
  announcer.destroy();
  strictEqual(dom.window.document.querySelector("[aria-live=polite]"), null);
});

Deno.test("IME composition is one undo group", async () => {
  await init();
  const { host } = fixture();
  const editor = Editor.fromJSON(PARAGRAPH);
  const mounted = mount(host, { state: editor, editor });
  ok(writeSelection(host, { from: 1, to: 1 }), "caret must be placed");
  const view = host.ownerDocument.defaultView!;
  host.dispatchEvent(
    new view.CompositionEvent("compositionstart", { bubbles: true }),
  );
  host.dispatchEvent(
    new view.CompositionEvent("compositionupdate", {
      data: "あ",
      bubbles: true,
    }),
  );
  deepStrictEqual(editor.isComposing(), true);
  host.dispatchEvent(
    new view.CompositionEvent("compositionend", { data: "あ", bubbles: true }),
  );
  deepStrictEqual(editor.isComposing(), false);
  deepStrictEqual(editor.doc().plainText(), "あOlá");
  deepStrictEqual(host.textContent, "あOlá");
  editor.undo();
  deepStrictEqual(editor.doc().plainText(), "Olá");
  mounted.destroy();
});

Deno.test("Mod+Home and Mod+End move the caret to document bounds", async () => {
  await init();
  const { host } = fixture();
  const editor = Editor.fromJSON(PARAGRAPH);
  const mounted = mount(host, { state: editor, editor });
  const view = host.ownerDocument.defaultView!;
  ok(writeSelection(host, { from: 2, to: 2 }), "caret must be placed");
  host.dispatchEvent(
    new view.KeyboardEvent("keydown", {
      key: "Home",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    }),
  );
  const atStart = editor.toJSON().selection;
  ok(atStart !== null && atStart.type === "caret", "caret expected");
  strictEqual(atStart.pos, 1);
  host.dispatchEvent(
    new view.KeyboardEvent("keydown", {
      key: "End",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    }),
  );
  const atEnd = editor.toJSON().selection;
  ok(atEnd !== null && atEnd.type === "caret", "caret expected");
  strictEqual(atEnd.pos, 5);
  mounted.destroy();
});
