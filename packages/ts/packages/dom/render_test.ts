import assert from "node:assert/strict";
import { renderTree } from "./render.ts";
import type { A11yNode } from "@rich-editor-auy/core";

class FakeText {
  readonly nodeType = 3;
  constructor(readonly text: string) {}
}

class FakeElement {
  readonly nodeType = 1;
  readonly children: Array<FakeElement | FakeText> = [];
  readonly attributes = new Map<string, string>();
  readonly dataset: Record<string, string> = {};
  textContent = "";

  constructor(
    readonly tagName: string,
    readonly ownerDocument: FakeDocument,
  ) {}

  append(...nodes: Array<FakeElement | FakeText>): void {
    this.children.push(...nodes);
  }

  replaceChildren(...nodes: Array<FakeElement | FakeText>): void {
    this.children.length = 0;
    this.children.push(...nodes);
  }

  setAttribute(name: string, value: string): void {
    this.attributes.set(name, value);
  }

  removeAttribute(name: string): void {
    this.attributes.delete(name);
  }
}

class FakeDocument {
  createElement(tag: string): FakeElement {
    return new FakeElement(tag, this);
  }

  createTextNode(text: string): FakeText {
    return new FakeText(text);
  }
}

function render(node: A11yNode): FakeElement {
  const doc = new FakeDocument();
  const host = new FakeElement("host", doc);
  renderTree(host as unknown as HTMLElement, {
    path: "",
    role: "document",
    range: { from: 0, to: 4 },
    children: [node],
  });
  return host;
}

Deno.test("renders native semantic elements without classes or styles", () => {
  const host = render({
    path: "0",
    role: "paragraph",
    range: { from: 0, to: 4 },
    children: [{
      path: "0/0",
      role: "text",
      text: "hello",
      marks: ["strong"],
      range: { from: 1, to: 4 },
      children: [],
    }],
  });
  const paragraph = host.children[0] as FakeElement;
  assert.equal(paragraph.tagName, "p");
  assert.ok(!paragraph.attributes.has("class"));
  assert.ok(!paragraph.attributes.has("style"));
  assert.equal(paragraph.dataset.auyRole, "paragraph");
  const strong = paragraph.children[0] as FakeElement;
  assert.equal(strong.tagName, "strong");
  assert.equal((strong.children[0] as FakeText).text, "hello");
});

Deno.test("renders heading level and list order", () => {
  const host = render({
    path: "0",
    role: "heading",
    level: 3,
    range: { from: 0, to: 4 },
    children: [{
      path: "0/0",
      role: "text",
      text: "Title",
      range: { from: 1, to: 4 },
      children: [],
    }],
  });
  const heading = host.children[0] as FakeElement;
  assert.equal(heading.tagName, "h3");
  assert.equal(heading.children.length, 1);
});

Deno.test("drops unsafe URL schemes", () => {
  const host = render({
    path: "0",
    role: "paragraph",
    range: { from: 0, to: 4 },
    children: [{
      path: "0/0",
      role: "text",
      text: "bad",
      marks: ["link"],
      href: "javascript:alert(1)",
      range: { from: 1, to: 4 },
      children: [],
    }],
  });
  const paragraph = host.children[0] as FakeElement;
  const link = paragraph.children[0] as FakeElement;
  assert.equal(link.attributes.has("href"), false);
});

Deno.test("renders links and images with accessible attributes", () => {
  const host = render({
    path: "0",
    role: "paragraph",
    range: { from: 0, to: 4 },
    children: [{
      path: "0/0",
      role: "text",
      text: "site",
      marks: ["link"],
      href: "https://example.test",
      range: { from: 1, to: 4 },
      children: [],
    }],
  });
  const paragraph = host.children[0] as FakeElement;
  const link = paragraph.children[0] as FakeElement;
  assert.equal(link.tagName, "a");
  assert.equal(link.attributes.get("href"), "https://example.test");
  assert(link.children.length === 1);
});

Deno.test("renders raw code language on block elements", () => {
  const host = render({
    path: "0",
    role: "code_block",
    code_language: "rust",
    range: { from: 0, to: 4 },
    children: [{
      path: "0/0",
      role: "text",
      text: "fn main() {}",
      range: { from: 1, to: 4 },
      children: [],
    }],
  });
  const pre = host.children[0] as FakeElement;
  assert.equal(pre.tagName, "pre");
  assert.equal(pre.dataset.auyLanguage, "rust");
  assert.equal(pre.dataset.auyRole, "code_block");
});

Deno.test("renders figures, image blocks, and table captions", () => {
  const host = render({
    path: "0",
    role: "figure",
    range: { from: 0, to: 4 },
    children: [{
      path: "0/0",
      role: "image",
      alt: "Diagram",
      src: "https://example.test/diagram.png",
      caption: "Flow",
      range: { from: 1, to: 2 },
      children: [],
    }, {
      path: "0/1",
      role: "caption",
      text: "Flow chart",
      range: { from: 2, to: 4 },
      children: [],
    }],
  });
  const figure = host.children[0] as FakeElement;
  assert.equal(figure.tagName, "figure");
  const image = figure.children[0] as FakeElement;
  assert.equal(image.tagName, "img");
  assert.equal(image.attributes.get("alt"), "Diagram");

  const tableHost = render({
    path: "0",
    role: "table",
    range: { from: 0, to: 4 },
    children: [{
      path: "0/0",
      role: "table_caption",
      range: { from: 0, to: 0 },
      children: [{
        path: "0/0/0",
        role: "text",
        text: "Sales",
        range: { from: 0, to: 0 },
        children: [],
      }],
    }],
  });
  const table = tableHost.children[0] as FakeElement;
  assert.equal(table.tagName, "table");
  assert.equal((table.children[0] as FakeElement).tagName, "caption");
});
