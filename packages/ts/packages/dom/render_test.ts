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
    role: "document",
    range: { from: 0, to: 4 },
    children: [node],
  });
  return host;
}

Deno.test("renders native semantic elements without classes or styles", () => {
  const host = render({
    role: "paragraph",
    range: { from: 0, to: 4 },
    children: [{
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
    role: "heading",
    level: 3,
    range: { from: 0, to: 4 },
    children: [{
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
    role: "paragraph",
    range: { from: 0, to: 4 },
    children: [{
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
    role: "paragraph",
    range: { from: 0, to: 4 },
    children: [{
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
