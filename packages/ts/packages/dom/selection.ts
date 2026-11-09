/**
 * Reading and writing the native DOM selection as a canonical document range.
 *
 * The adapter never derives semantics from the DOM: it converts the native selection into byte
 * positions and hands those to the core, and writes core positions back into the native selection
 * so screen readers and the browser keep working.
 */

import { pointAt, positionAt } from "./positions.ts";

/** Byte range in the canonical document. */
export interface DocumentRange {
  readonly from: number;
  readonly to: number;
}

/** Returns whether a node belongs to the rendered tree. */
function inside(root: HTMLElement, node: Node | null): boolean {
  return node !== null && (node === root || root.contains(node));
}

/** Reads the native selection as a document range. */
export function readSelection(root: HTMLElement): DocumentRange | undefined {
  const selection = root.ownerDocument.getSelection();
  if (selection === null || selection.rangeCount === 0) return undefined;
  const { anchorNode, anchorOffset, focusNode, focusOffset } = selection;
  if (
    anchorNode === null || focusNode === null ||
    !inside(root, anchorNode) || !inside(root, focusNode)
  ) {
    return undefined;
  }
  const anchor = positionAt({ node: anchorNode, offset: anchorOffset });
  const focus = positionAt({ node: focusNode, offset: focusOffset });
  if (anchor === undefined || focus === undefined) return undefined;
  return {
    from: Math.min(anchor, focus),
    to: Math.max(anchor, focus),
  };
}

/** Writes a document range into the native selection. */
export function writeSelection(
  root: HTMLElement,
  range: DocumentRange,
): boolean {
  const doc = root.ownerDocument;
  const anchor = pointAt(root, range.from);
  const focus = pointAt(root, range.to);
  if (anchor === undefined || focus === undefined) return false;
  const selection = doc.getSelection();
  if (selection === null) return false;
  const next = doc.createRange();
  next.setStart(anchor.node, anchor.offset);
  next.setEnd(focus.node, focus.offset);
  selection.removeAllRanges();
  selection.addRange(next);
  return true;
}
