import type { A11yNode } from "@rich-editor-auy/core";

/** How rendered elements expose their semantics. */
export interface RenderOptions {
  /**
   * `native` uses `<h1>`, `<ul>`, `<a>` and `<img alt>`; `neutral` uses `div`/`span` plus the
   * role, name and level the semantic tree declares.
   */
  readonly semantics?: "native" | "neutral";
}

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
export function renderTree(
  host: HTMLElement,
  tree: A11yNode,
  options: RenderOptions = {},
): void {
  const doc = host.ownerDocument;
  const mode = options.semantics ?? "native";
  host.replaceChildren(
    ...tree.children.map((child) => renderNode(doc, child, mode)),
  );
}

function renderNode(
  doc: Document,
  node: A11yNode,
  mode: "native" | "neutral",
): Node {
  if (node.role === "text") {
    return renderText(doc, node, mode);
  }
  const element = mode === "neutral"
    ? neutralElement(doc, node)
    : createElement(doc, node);
  applyAttributes(element, node, mode);
  if (node.role === "image") {
    return element;
  }
  if (node.children.length > 0) {
    element.append(
      ...node.children.map((child) => renderNode(doc, child, mode)),
    );
  } else if (node.text !== undefined) {
    element.textContent = node.text;
  }
  return element;
}

/** Builds the neutral element for a role, with ARIA derived from the tree. */
function neutralElement(doc: Document, node: A11yNode): HTMLElement {
  const element = doc.createElement("div");
  element.setAttribute("role", neutralRole(node.role));
  if (node.role === "heading") {
    element.setAttribute("aria-level", String(node.level ?? 2));
  }
  return element;
}

/** Maps a semantic role onto the ARIA role of the neutral mode. */
function neutralRole(role: string): string {
  switch (role) {
    case "paragraph":
      return "paragraph";
    case "list_item":
      return "listitem";
    case "table_row":
      return "row";
    case "table_header":
    case "table_cell":
      return "cell";
    case "table_caption":
    case "caption":
      return "caption";
    case "code_block":
    case "code":
      return "code";
    case "quote":
      return "blockquote";
    case "separator":
      return "separator";
    case "custom":
      return "group";
    default:
      return role;
  }
}

function renderText(
  doc: Document,
  node: A11yNode,
  mode: "native" | "neutral",
): Node {
  const text = node.text ?? node.name ?? "";
  let current: Node = doc.createTextNode(text);
  const marks = node.marks ?? [];
  for (const mark of [...marks].reverse()) {
    const tag = MARK_TAGS[mark];
    if (tag === undefined) continue;
    const wrapper = mode === "neutral"
      ? doc.createElement("span")
      : doc.createElement(tag);
    wrapper.dataset.auyMark = mark;
    wrapper.append(current);
    const href = node.href === undefined ? undefined : safeUrl(node.href);
    if (tag === "a" && href !== undefined && mode === "native") {
      wrapper.setAttribute("href", href);
    }
    current = wrapper;
  }
  const href = node.href === undefined ? undefined : safeUrl(node.href);
  if (href !== undefined && !marks.includes("link")) {
    const link = mode === "neutral"
      ? neutralElement(doc, { ...node, role: "link", href })
      : doc.createElement("a");
    if (mode === "neutral") {
      link.setAttribute("role", "link");
    } else {
      link.setAttribute("href", href);
    }
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

function applyAttributes(
  element: HTMLElement,
  node: A11yNode,
  mode: "native" | "neutral",
): void {
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
  if (node.code_language !== undefined) {
    element.dataset.auyLanguage = node.code_language;
  }
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
  if (mode === "neutral" && node.role === "image") {
    element.setAttribute("role", "img");
  }
  if (mode === "neutral" && node.role !== "table_caption") {
    const label = node.alt ?? node.name;
    if (label !== undefined && label.length > 0) {
      element.setAttribute("aria-label", label);
    }
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
