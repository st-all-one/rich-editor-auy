import type {
  A11yNode,
  BatchPayload,
  EditorLike,
  SliceJSON,
  StateProvider,
} from "@rich-editor-auy/core";
import { type Announcer, createAnnouncer, describeChange } from "./announce.ts";
import { bindInput, type EditorIntent } from "./input.ts";
import type { RenderOptions } from "./render.ts";
import { renderTree } from "./render.ts";
import { readSelection, writeSelection } from "./selection.ts";

export interface MountOptions {
  readonly state: StateProvider;
  /**
   * Editable surface. Use `Editor` (or any `EditorLike`) to let the adapter apply intents, or
   * `onIntent` to handle them in the host.
   */
  readonly editor?: EditorLike;
  readonly editable?: boolean;
  /** `native` (default) renders semantic elements; `neutral` renders `div`/`span` plus ARIA. */
  readonly semantics?: "native" | "neutral";
  readonly label?: string;
  /** Announces remote changes and errors in a polite live region. */
  readonly live?: boolean;
  readonly onInput?: (event: InputEvent, state: StateProvider) => void;
  readonly onIntent?: (intent: EditorIntent) => void;
}

export interface MountedEditor {
  destroy(): void;
  view(): HTMLElement;
  dispatch(state: StateProvider): void;
  state(): StateProvider;
  /** Re-applies the core selection to the native selection. */
  syncSelection(): void;
}

interface SavedHost {
  readonly html: string;
  readonly editable: string | null;
  readonly role: string | null;
  readonly multiline: string | null;
  readonly lang: string | null;
  readonly dir: string | null;
  readonly label: string | null;
}

/** Mounts a semantic editor shell over a consumer-owned host element. */
export function mount(host: HTMLElement, options: MountOptions): MountedEditor {
  const saved: SavedHost = {
    html: host.innerHTML,
    editable: host.getAttribute("contenteditable"),
    role: host.getAttribute("role"),
    multiline: host.getAttribute("aria-multiline"),
    lang: host.getAttribute("lang"),
    dir: host.getAttribute("dir"),
    label: host.getAttribute("aria-label"),
  };
  const editable = options.editable !== false;
  host.setAttribute("role", "textbox");
  host.setAttribute("aria-multiline", "true");
  host.setAttribute("contenteditable", editable ? "true" : "false");
  host.dataset.auyRoot = "true";
  let state = options.state;
  const renderOptions: RenderOptions = { semantics: options.semantics };
  const announcer: Announcer | undefined = options.live === false
    ? undefined
    : createAnnouncer(host);
  let syncing = false;

  const render = (): void => {
    const tree: A11yNode = state.a11yTree();
    host.setAttribute("aria-label", options.label ?? "Document");
    if (tree.lang !== undefined) host.setAttribute("lang", tree.lang);
    if (tree.dir !== undefined) host.setAttribute("dir", tree.dir);
    renderTree(host, tree, renderOptions);
  };

  const dispatch = (next: StateProvider): void => {
    state = next;
    render();
    syncSelection();
  };

  const syncSelection = (): void => {
    const range = readSelection(host);
    if (range === undefined) return;
    const applied = writeSelection(host, range);
    if (!applied || options.editor === undefined) return;
    if (syncing) return;
    syncing = true;
    try {
      options.editor.setSelection(range.from, range.to);
    } finally {
      syncing = false;
    }
  };

  const onSelectionChange = (): void => {
    if (!syncing) syncSelection();
  };

  const onInput = (event: Event): void => {
    if (options.onInput !== undefined) {
      options.onInput(event as InputEvent, state);
    }
  };

  const onIntent = (intent: EditorIntent): void => {
    if (options.onIntent !== undefined) {
      options.onIntent(intent);
      return;
    }
    if (options.editor === undefined) return;
    applyIntent(options.editor, intent);
    dispatch(options.editor);
    if (intent.kind === "move_selection") {
      writeSelection(host, { from: intent.from, to: intent.to });
    }
  };

  const unbindInput = editable ? bindInput(host, { onIntent }) : () => {};

  host.addEventListener("input", onInput);
  host.ownerDocument.addEventListener("selectionchange", onSelectionChange);
  render();

  return {
    destroy(): void {
      host.removeEventListener("input", onInput);
      host.ownerDocument.removeEventListener(
        "selectionchange",
        onSelectionChange,
      );
      unbindInput();
      announcer?.destroy();
      host.innerHTML = saved.html;
      restore(host, "contenteditable", saved.editable);
      restore(host, "role", saved.role);
      restore(host, "aria-multiline", saved.multiline);
      restore(host, "lang", saved.lang);
      restore(host, "dir", saved.dir);
      restore(host, "aria-label", saved.label);
      delete host.dataset.auyRoot;
    },
    view(): HTMLElement {
      return host;
    },
    dispatch,
    state(): StateProvider {
      return state;
    },
    syncSelection,
  };
}

/** Applies one intent through the core editor. */
function applyIntent(
  editor: EditorLike,
  intent: EditorIntent,
): unknown {
  switch (intent.kind) {
    case "insert_text":
      return editor.apply(
        batch({ op: "insert_text", at: intent.at, text: intent.text }),
      );
    case "delete":
      return editor.apply(batch({
        op: "delete_range",
        from: intent.from,
        to: intent.to,
      }));
    case "split":
      return editor.apply(batch({ op: "split_block", at: intent.at }));
    case "join":
      return editor.apply(batch({ op: "join_backward", at: intent.at }));
    case "toggle_mark":
      return editor.apply(batch({
        op: "toggle_mark",
        from: intent.from,
        to: intent.to,
        mark: intent.mark,
      }));
    case "paste":
      return intent.from === intent.to
        ? editor.pasteSlice(intent.at, intent.slice)
        : editor.replaceSlice(intent.from, intent.to, intent.slice);
    case "undo":
      return editor.undo();
    case "redo":
      return editor.redo();
    case "move_selection":
      return editor.setSelection(intent.from, intent.to);
    case "composition_start":
      return editor.beginComposition(intent.from, intent.to);
    case "composition_update":
      return editor.updateComposition(intent.text);
    case "composition_commit":
      return editor.commitComposition();
    case "composition_cancel":
      return editor.cancelComposition();
  }
}

/** Builds a one-operation batch payload. */
function batch(op: Record<string, unknown>): BatchPayload {
  return { ops: [op] } as unknown as BatchPayload;
}

/** Alias for hosts that prefer explicit binding terminology. */
export function bind(host: HTMLElement, options: MountOptions): MountedEditor {
  return mount(host, options);
}

function restore(host: HTMLElement, name: string, value: string | null): void {
  if (value === null) host.removeAttribute(name);
  else host.setAttribute(name, value);
}

export { describeChange, type SliceJSON };
