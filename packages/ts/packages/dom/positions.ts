/**
 * Mapping between rendered DOM points and canonical document positions.
 *
 * Core positions are UTF-8 byte offsets plus one structural token per block edge; DOM offsets are
 * UTF-16 code units inside a text node. The rendered tree exposes `data-auy-from`/`data-auy-to` on
 * every semantic element, so a text node inherits the range of its nearest rendered ancestor and
 * the byte distance inside that block follows from the text itself.
 */

const encoder = new TextEncoder();

/** Returns the UTF-8 byte length of a string. */
export function byteLength(text: string): number {
  return encoder.encode(text).length;
}

/** One text node of the rendered tree with its absolute byte range. */
export interface TextSpan {
  readonly node: Text;
  readonly start: number;
  readonly end: number;
}

/** A DOM point relative to the rendered tree. */
export interface DomPoint {
  readonly node: Node;
  readonly offset: number;
}

/** Returns the nearest rendered ancestor that carries a document range. */
export function rangeElement(node: Node): HTMLElement | undefined {
  let current: Node | null = node;
  while (current !== null) {
    if (current.nodeType === 1) {
      const element = current as HTMLElement;
      if (element.dataset.auyFrom !== undefined) return element;
    }
    current = current.parentNode;
  }
  return undefined;
}

/** Collects the text spans of the rendered tree in reading order. */
export function textSpans(root: HTMLElement): TextSpan[] {
  const spans: TextSpan[] = [];
  const walker = root.ownerDocument.createTreeWalker(root, 4);
  let block: HTMLElement | undefined;
  let cursor = 0;
  for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
    const text = node as Text;
    const owner = rangeElement(text);
    if (owner !== block) {
      block = owner;
      cursor = owner === undefined ? 0 : contentStart(owner);
    }
    const length = byteLength(text.data);
    if (length > 0) {
      spans.push({ node: text, start: cursor, end: cursor + length });
      cursor += length;
    }
  }
  return spans;
}

/** Returns the first content position of a rendered block. */
function contentStart(element: HTMLElement): number {
  const from = Number(element.dataset.auyFrom ?? "0");
  return from + 1;
}

/** Returns the document size projected by the rendered tree. */
export function documentSize(root: HTMLElement): number {
  let size = 0;
  for (const element of root.querySelectorAll<HTMLElement>("[data-auy-to]")) {
    size = Math.max(size, Number(element.dataset.auyTo ?? "0"));
  }
  return size;
}

/** Returns the DOM point for a document position. */
export function pointAt(
  root: HTMLElement,
  position: number,
): DomPoint | undefined {
  for (const span of textSpans(root)) {
    if (position >= span.start && position <= span.end) {
      return {
        node: span.node,
        offset: offsetForBytes(span.node.data, position - span.start),
      };
    }
  }
  return blockPointAt(root, position);
}

/** Returns a point on the smallest rendered block that contains the position. */
function blockPointAt(
  root: HTMLElement,
  position: number,
): DomPoint | undefined {
  let best: HTMLElement | undefined;
  let size = Number.POSITIVE_INFINITY;
  for (const element of root.querySelectorAll<HTMLElement>("[data-auy-from]")) {
    const from = Number(element.dataset.auyFrom ?? "0");
    const to = Number(element.dataset.auyTo ?? "0");
    if (position >= from && position <= to && to - from < size) {
      best = element;
      size = to - from;
    }
  }
  if (best === undefined) return undefined;
  const from = Number(best.dataset.auyFrom ?? "0");
  return {
    node: best,
    offset: position <= from + 1 ? 0 : best.childNodes.length,
  };
}

/** Returns the document position for a DOM point. */
export function positionAt(point: DomPoint): number | undefined {
  if (point.node.nodeType === 3) {
    const text = point.node as Text;
    const owner = rangeElement(text);
    if (owner === undefined) return undefined;
    return contentStart(owner) + bytesBefore(owner, text, point.offset);
  }
  const element = point.node as HTMLElement;
  const from = element.dataset?.auyFrom;
  if (from !== undefined) {
    return point.offset <= 0 ? Number(from) + 1 : contentEnd(element);
  }
  return undefined;
}

/** Returns the byte distance from the block content start to a DOM offset. */
function bytesBefore(
  block: HTMLElement,
  target: Text,
  offset: number,
): number {
  let total = 0;
  const walker = block.ownerDocument.createTreeWalker(block, 4);
  for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
    const text = node as Text;
    if (text === target) return total + byteLength(text.data.slice(0, offset));
    total += byteLength(text.data);
  }
  return total;
}

/** Returns the last content position of a rendered block. */
function contentEnd(element: HTMLElement): number {
  const to = Number(element.dataset?.auyTo ?? "0");
  return to > 0 ? to - 1 : to;
}

/** Converts a byte distance inside a string to a UTF-16 offset. */
export function offsetForBytes(text: string, bytes: number): number {
  let used = 0;
  let offset = 0;
  for (const character of text) {
    const size = byteLength(character);
    if (used + size > bytes) {
      // A byte offset inside a code point rounds forward, matching caret placement.
      if (used < bytes) offset += character.length;
      break;
    }
    used += size;
    offset += character.length;
  }
  return offset;
}
