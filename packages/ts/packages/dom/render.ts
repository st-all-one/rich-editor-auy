import type { A11yNode } from "@rich-editor-auy/core";

/** Native element selected for a semantic role. */
export interface RenderTargets {
  readonly document: Document;
}

const MARK_TAGS: Readonly<Record<string, string>> = {
  strong: "strong",
  em: "em",
  code: "code",
  link: "a",
  underline: "u",
  strike: "s",
  highlight: "mark",
  sup: "sup",
  sub: "sub",
};

/** Renders one semantic tree into a host without classes, styles, or wrappers. */
export function renderTree(host: HTMLElement, tree: A11yNode): void {
  const doc = host.ownerDocument;
  host.replaceChildren(...tree.children.map((child) => renderNode(doc, child)));
}

function renderNode(doc: Document, node: A11yNode): Node {
  if (node.role === "text") {
    return renderText(doc, node);
  }
  const element = createElement(doc, node);
  applyAttributes(element, node);
  if (node.role === "image") {
    return element;
  }
  if (node.children.length > 0) {
    element.append(...node.children.map((child) => renderNode(doc, child)));
  } else if (node.text !== undefined) {
    element.textContent = node.text;
  }
  return element;
}

function renderText(doc: Document, node: A11yNode): Node {
  const text = node.text ?? node.name ?? "";
  let current: Node = doc.createTextNode(text);
  const marks = node.marks ?? [];
  for (const mark of [...marks].reverse()) {
    const tag = MARK_TAGS[mark];
    if (tag === undefined) continue;
    const wrapper = doc.createElement(tag);
    wrapper.append(current);
    const href = node.href === undefined ? undefined : safeUrl(node.href);
    if (tag === "a" && href !== undefined) {
      wrapper.setAttribute("href", href);
    }
    current = wrapper;
  }
  const href = node.href === undefined ? undefined : safeUrl(node.href);
  if (href !== undefined && !marks.includes("link")) {
    const link = doc.createElement("a");
    link.setAttribute("href", href);
    link.append(current);
    current = link;
  }
  return current;
}

function createElement(doc: Document, node: A11yNode): HTMLElement {
  switch (node.role) {
    case "paragraph":
      return doc.createElement("p");
    case "heading": {
      const level = Math.min(6, Math.max(1, node.level ?? 2));
      return doc.createElement(`h${level}`);
    }
    case "list":
      return doc.createElement(node.ordered === true ? "ol" : "ul");
    case "list_item":
      return doc.createElement("li");
    case "quote":
      return doc.createElement("blockquote");
    case "code":
      return doc.createElement("code");
    case "code_block":
      return doc.createElement("pre");
    case "table":
      return doc.createElement("table");
    case "table_caption":
      return doc.createElement("caption");
    case "table_row":
      return doc.createElement("tr");
    case "table_header":
      return doc.createElement("th");
    case "table_cell":
      return doc.createElement("td");
    case "link":
      return doc.createElement("a");
    case "image":
      return doc.createElement("img");
    case "figure":
      return doc.createElement("figure");
    case "caption":
      return doc.createElement("figcaption");
    case "video":
      return doc.createElement("video");
    case "audio":
      return doc.createElement("audio");
    case "attachment":
      return doc.createElement("a");
    case "separator":
      return doc.createElement("hr");
    default:
      return doc.createElement("div");
  }
}

function applyAttributes(element: HTMLElement, node: A11yNode): void {
  element.dataset.auyRole = node.role;
  element.dataset.auyFrom = String(node.range.from);
  element.dataset.auyTo = String(node.range.to);
  if (node.lang !== undefined) element.setAttribute("lang", node.lang);
  if (node.dir !== undefined) element.setAttribute("dir", node.dir);
  const href = node.href === undefined ? undefined : safeUrl(node.href);
  if (href !== undefined) element.setAttribute("href", href);
  const src = node.src === undefined ? undefined : safeUrl(node.src);
  if (src !== undefined) element.setAttribute("src", src);
  if (node.alt !== undefined) element.setAttribute("alt", node.alt);
  if (node.caption !== undefined) element.setAttribute("title", node.caption);
  if (node.colspan !== undefined) {
    element.setAttribute("colspan", String(node.colspan));
  }
  if (node.rowspan !== undefined) {
    element.setAttribute("rowspan", String(node.rowspan));
  }
  if (node.scope !== undefined) element.setAttribute("scope", node.scope);
  if (node.role === "video" || node.role === "audio") {
    element.setAttribute("controls", "");
  }
  if (node.role === "attachment") {
    element.setAttribute("download", "");
  }
  if (node.role === "custom") {
    element.setAttribute("role", "group");
  }
}

function safeUrl(value: string): string | undefined {
  const scheme = /^([a-z][a-z0-9+.-]*):/i.exec(value);
  if (scheme === null) return value;
  return ["http", "https", "mailto", "tel"].includes(scheme[1].toLowerCase())
    ? value
    : undefined;
}
