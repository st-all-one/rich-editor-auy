/**
 * Input intents: DOM events translated into core operations.
 *
 * The adapter is an intent boundary. It reads the native event, computes a document position from
 * the rendered ranges, and asks the host or the core editor to perform the edit. It never edits
 * the DOM itself and never reimplements the edit algebra.
 */

import type { SliceJSON } from "@rich-editor-auy/core";
import { documentSize } from "./positions.ts";
import { readSelection } from "./selection.ts";

/** What the user asked for, expressed in document terms. */
export type EditorIntent =
  | { readonly kind: "insert_text"; readonly at: number; readonly text: string }
  | {
    readonly kind: "delete";
    readonly from: number;
    readonly to: number;
    readonly direction: "backward" | "forward";
  }
  | { readonly kind: "split"; readonly at: number }
  | { readonly kind: "join"; readonly at: number }
  | {
    readonly kind: "paste";
    readonly at: number;
    readonly slice: SliceJSON;
    readonly from: number;
    readonly to: number;
  }
  | { readonly kind: "undo" }
  | { readonly kind: "redo" }
  | {
    readonly kind: "move_selection";
    readonly from: number;
    readonly to: number;
  }
  | {
    readonly kind: "composition_start";
    readonly from: number;
    readonly to: number;
  }
  | { readonly kind: "composition_update"; readonly text: string }
  | { readonly kind: "composition_commit" }
  | { readonly kind: "composition_cancel" }
  | {
    readonly kind: "toggle_mark";
    readonly mark: string;
    readonly from: number;
    readonly to: number;
  };

/** Options for {@link bindInput}. */
export interface InputOptions {
  /** Receives every translated intent. */
  readonly onIntent: (intent: EditorIntent) => void;
}

/** Binds the input surface and returns its cleanup. */
export function bindInput(
  host: HTMLElement,
  options: InputOptions,
): () => void {
  let composing = false;
  const onBeforeInput = (event: Event): void => {
    const input = event as InputEvent;
    // Composed text arrives through the composition events, not as separate edits.
    if (composing || input.inputType.startsWith("insertComposition")) return;
    const intent = intentOf(host, input);
    if (intent === undefined) return;
    event.preventDefault();
    options.onIntent(intent);
  };
  const onCompositionStart = (): void => {
    composing = true;
    const range = readSelection(host);
    if (range === undefined) return;
    options.onIntent({
      kind: "composition_start",
      from: range.from,
      to: range.to,
    });
  };
  const onCompositionUpdate = (event: Event): void => {
    const data = (event as CompositionEvent).data ?? "";
    options.onIntent({ kind: "composition_update", text: data });
  };
  const onCompositionEnd = (event: Event): void => {
    composing = false;
    const data = (event as CompositionEvent).data ?? "";
    options.onIntent(
      data.length === 0
        ? { kind: "composition_cancel" }
        : { kind: "composition_commit" },
    );
  };
  const onKeyDown = (event: Event): void => {
    const keyboard = event as KeyboardEvent;
    const intent = shortcutOf(keyboard, host);
    if (intent === undefined) return;
    event.preventDefault();
    options.onIntent(intent);
  };
  const onPaste = (event: Event): void => {
    const clipboard = event as ClipboardEvent;
    const range = readSelection(host);
    const intent = pasteIntent(clipboard, range, documentSize(host));
    if (intent === undefined) return;
    event.preventDefault();
    options.onIntent(intent);
  };
  host.addEventListener("beforeinput", onBeforeInput);
  host.addEventListener("keydown", onKeyDown);
  host.addEventListener("paste", onPaste);
  host.addEventListener("compositionstart", onCompositionStart);
  host.addEventListener("compositionupdate", onCompositionUpdate);
  host.addEventListener("compositionend", onCompositionEnd);
  return () => {
    host.removeEventListener("beforeinput", onBeforeInput);
    host.removeEventListener("keydown", onKeyDown);
    host.removeEventListener("paste", onPaste);
    host.removeEventListener("compositionstart", onCompositionStart);
    host.removeEventListener("compositionupdate", onCompositionUpdate);
    host.removeEventListener("compositionend", onCompositionEnd);
  };
}

/** Translates one `beforeinput` event. */
function intentOf(
  host: HTMLElement,
  event: InputEvent,
): EditorIntent | undefined {
  const size = documentSize(host);
  const range = readSelection(host);
  if (range === undefined) return undefined;
  switch (event.inputType) {
    case "insertText":
    case "insertReplacementText": {
      if (range.from !== range.to) {
        return {
          kind: "delete",
          from: range.from,
          to: range.to,
          direction: "forward",
        };
      }
      const text = event.data ?? "";
      return text.length === 0
        ? undefined
        : { kind: "insert_text", at: range.from, text };
    }
    case "insertParagraph":
    case "insertLineBreak":
      return { kind: "split", at: range.to };
    case "deleteContentBackward":
    case "deleteWordBackward":
      return {
        kind: "delete",
        from: deletionBefore(host, range),
        to: range.to,
        direction: "backward",
      };
    case "deleteContentForward":
    case "deleteWordForward":
      return {
        kind: "delete",
        from: range.from,
        to: deletionAfter(host, range, size),
        direction: "forward",
      };
    case "historyUndo":
      return { kind: "undo" };
    case "historyRedo":
      return { kind: "redo" };
    case "formatBold":
      return {
        kind: "toggle_mark",
        mark: "strong",
        from: range.from,
        to: range.to,
      };
    case "formatItalic":
      return {
        kind: "toggle_mark",
        mark: "em",
        from: range.from,
        to: range.to,
      };
    default:
      return undefined;
  }
}

/** Resolves the start of the deletion before the caret. */
function deletionBefore(
  host: HTMLElement,
  range: { readonly from: number; readonly to: number },
): number {
  if (range.from !== range.to) return range.from;
  return Math.min(range.to, Math.max(0, caretStep(host, range.from, false)));
}

/** Resolves the end of the deletion after the caret. */
function deletionAfter(
  host: HTMLElement,
  range: { readonly from: number; readonly to: number },
  size: number,
): number {
  if (range.from !== range.to) return range.to;
  return Math.min(size, Math.max(0, caretStep(host, range.from, true)));
}

/** Returns the position one character away from the caret, in UTF-8 bytes. */
function caretStep(host: HTMLElement, at: number, forward: boolean): number {
  const selection = host.ownerDocument.getSelection();
  const node = selection?.anchorNode ?? null;
  if (node === null || node.nodeType !== 3) return at + (forward ? 1 : -1);
  const text = (node as Text).data;
  const chunk = forward
    ? text.slice(selection?.anchorOffset ?? 0)
    : text.slice(0, selection?.anchorOffset ?? 0);
  const character = forward
    ? Array.from(chunk).shift()
    : Array.from(chunk).pop();
  if (character === undefined) return at;
  const bytes = new TextEncoder().encode(character).length;
  return forward ? at + bytes : at - bytes;
}

/** Translates a keyboard shortcut into an intent. */
function shortcutOf(
  event: KeyboardEvent,
  host: HTMLElement,
): EditorIntent | undefined {
  const range = readSelection(host);
  if (range === undefined) return undefined;
  const mod = event.metaKey || event.ctrlKey;
  if (!mod) return undefined;
  if (event.key === "z" && !event.shiftKey) return { kind: "undo" };
  if ((event.key === "z" && event.shiftKey) || event.key === "y") {
    return { kind: "redo" };
  }
  if (event.key === "b") {
    return {
      kind: "toggle_mark",
      mark: "strong",
      from: range.from,
      to: range.to,
    };
  }
  if (event.key === "i") {
    return {
      kind: "toggle_mark",
      mark: "em",
      from: range.from,
      to: range.to,
    };
  }
  if (event.key === "Home" || event.key === "End") {
    const size = documentSize(host);
    const at = event.key === "Home" ? Math.min(1, size) : Math.max(0, size - 1);
    return { kind: "move_selection", from: at, to: at };
  }
  if (event.key === "Backspace" && range.from === range.to && range.from > 0) {
    return { kind: "join", at: range.from };
  }
  return undefined;
}

/** Translates a paste event by picking the richest supported format. */
function pasteIntent(
  event: ClipboardEvent,
  range: { readonly from: number; readonly to: number } | undefined,
  size: number,
): EditorIntent | undefined {
  const data = event.clipboardData;
  if (data === null || range === undefined) return undefined;
  const at = Math.min(range.to, size);
  const json = data.getData("application/x-rich-editor-auy+json");
  if (json.length > 0) {
    try {
      const slice = JSON.parse(json) as SliceJSON;
      return { kind: "paste", at, slice, from: range.from, to: range.to };
    } catch {
      // Fall through to plain text: a malformed payload never blocks the paste.
    }
  }
  const text = data.getData("text/plain");
  if (text.length === 0) return undefined;
  return {
    kind: "paste",
    at,
    slice: { text },
    from: range.from,
    to: range.to,
  };
}
